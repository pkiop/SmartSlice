use ffmpeg_next as ffmpeg;
use std::env;

pub fn read_video() -> Result<(), Box<dyn std::error::Error>> {
    // FFmpeg 초기화
    ffmpeg::init()?;

    // 명령행 인자에서 비디오 파일 경로 가져오기
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("사용법: cargo run <비디오_파일_경로>");
        return Ok(());
    }

    let input_path = &args[1];
    println!("비디오 파일 읽기: {}", input_path);

    // 입력 파일 열기
    let mut input = ffmpeg::format::input(&input_path)?;

    // 파일 정보 출력
    println!("=== 파일 정보 ===");
    println!("포맷: {}", input.format().name());
    println!("지속시간: {:?}", input.duration());
    println!("비트레이트: {}", input.bit_rate());
    println!("스트림 개수: {}", input.streams().count());

    // 각 스트림 정보 출력
    for (index, stream) in input.streams().enumerate() {
        println!("\n=== 스트림 {} ===", index);
        println!("타입: {:?}", stream.parameters().medium());
        println!("코덱: {:?}", stream.parameters().id());
        println!("지속시간: {:?}", stream.duration());
        println!("프레임 수: {}", stream.frames());
        println!("시간 기준: {:?}", stream.time_base());

        // 비디오 스트림인 경우 추가 정보
        if stream.parameters().medium() == ffmpeg::media::Type::Video {
            let ctx =
                ffmpeg::codec::context::Context::from_parameters(stream.parameters()).unwrap();
            let video = ctx.decoder().video().unwrap();
            println!("해상도: {}x{}", video.width(), video.height());
            println!("프레임레이트: {:?}", stream.avg_frame_rate());
            println!("픽셀 포맷: {:?}", video.format());
        }

        // 오디오 스트림인 경우 추가 정보
        if stream.parameters().medium() == ffmpeg::media::Type::Audio {
            let ctx =
                ffmpeg::codec::context::Context::from_parameters(stream.parameters()).unwrap();
            let audio = ctx.decoder().audio().unwrap();
            // println!("샘플레이트: {}", audio.sample_rate());
            println!("채널 수: {}", audio.channels());
            println!("샘플 포맷: {:?}", audio.format());
        }
    }

    // 비디오 스트림 찾기
    let video_stream_index = input
        .streams()
        .best(ffmpeg::media::Type::Video)
        .map(|stream| stream.index());

    if let Some(video_index) = video_stream_index {
        println!("\n=== 비디오 프레임 읽기 ===");
        read_video_frames(&mut input, video_index)?;
    } else {
        println!("비디오 스트림을 찾을 수 없습니다.");
    }

    Ok(())
}

fn read_video_frames(
    input: &mut ffmpeg::format::context::Input,
    video_stream_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // 비디오 스트림 가져오기
    let video_stream = input.stream(video_stream_index).unwrap();

    // 디코더 생성
    let context_decoder =
        ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    // 프레임 카운터
    let mut frame_count = 0;
    let max_frames = 10; // 처음 10프레임만 읽기

    // 패킷 읽기
    for (stream, packet) in input.packets() {
        if stream.index() == video_stream_index {
            // 패킷을 디코더로 전송
            decoder.send_packet(&packet)?;

            // 디코딩된 프레임 받기
            let mut frame = ffmpeg::util::frame::Video::empty();
            while decoder.receive_frame(&mut frame).is_ok() {
                println!(
                    "프레임 {}: {}x{}, 포맷: {:?}, PTS: {:?}",
                    frame_count,
                    frame.width(),
                    frame.height(),
                    frame.format(),
                    frame.pts()
                );

                frame_count += 1;
                if frame_count >= max_frames {
                    println!("{}개 프레임 읽기 완료", max_frames);
                    return Ok(());
                }
            }
        }
    }

    // 디코더 플러시 (남은 프레임 처리)
    decoder.send_eof()?;
    let mut frame = ffmpeg::util::frame::Video::empty();
    while decoder.receive_frame(&mut frame).is_ok() {
        println!(
            "프레임 {}: {}x{}, 포맷: {:?}, PTS: {:?}",
            frame_count,
            frame.width(),
            frame.height(),
            frame.format(),
            frame.pts()
        );

        frame_count += 1;
        if frame_count >= max_frames {
            break;
        }
    }

    println!("총 {}개 프레임 읽음", frame_count);
    Ok(())
}

// 비디오 썸네일 추출 함수
pub fn extract_thumbnail(
    input_path: &str,
    output_path: &str,
    timestamp: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg::init()?;

    let mut input = ffmpeg::format::input(input_path)?;
    let video_stream = input
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or("비디오 스트림을 찾을 수 없습니다")?;

    let video_stream_index = video_stream.index();

    // 디코더 설정
    let context_decoder =
        ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    // 특정 시간으로 시크
    let time_base = video_stream.time_base();
    let seek_timestamp = (timestamp / time_base.0 as f64 * time_base.1 as f64) as i64;

    // 시크 수행 (키프레임으로)
    input.seek(seek_timestamp, ..seek_timestamp)?;

    // 프레임 읽기
    for (stream, packet) in input.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;

            let mut frame = ffmpeg::util::frame::Video::empty();
            if decoder.receive_frame(&mut frame).is_ok() {
                println!(
                    "썸네일 추출 완료: {}x{} at {}초",
                    frame.width(),
                    frame.height(),
                    timestamp
                );

                // 여기서 실제 이미지 저장 로직을 구현해야 함
                // 예: frame을 PNG나 JPEG로 저장
                break;
            }
        }
    }

    Ok(())
}

// 비디오 정보만 간단히 가져오는 함수
pub fn get_video_info(input_path: &str) -> Result<VideoInfo, Box<dyn std::error::Error>> {
    ffmpeg::init()?;

    let input = ffmpeg::format::input(input_path)?;
    let video_stream = input
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or("비디오 스트림을 찾을 수 없습니다")?;

    let duration_seconds = video_stream.duration() as f64 * video_stream.time_base().0 as f64
        / video_stream.time_base().1 as f64;

    let ctx = ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())?;
    let video = ctx.decoder().video()?;

    Ok(VideoInfo {
        width: video.width(),
        height: video.height(),
        duration: duration_seconds,
        frame_rate: video_stream.avg_frame_rate().0 as f64 / video_stream.avg_frame_rate().1 as f64,
        codec: format!("{:?}", video_stream.parameters().id()),
        format: input.format().name().to_string(),
        bit_rate: input.bit_rate() as usize,
    })
}

#[derive(Debug)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub duration: f64,
    pub frame_rate: f64,
    pub codec: String,
    pub format: String,
    pub bit_rate: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_info() {
        // 테스트용 비디오 파일이 있다면
        // let info = get_video_info("test_video.mp4").unwrap();
        // assert!(info.width > 0);
        // assert!(info.height > 0);
    }
}
