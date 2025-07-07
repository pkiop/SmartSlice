extern crate ffmpeg_next as ffmpeg;

mod example;

fn main() {
    // 기본 비디오 읽기
    if let Err(e) = example::read_video::read_video() {
        eprintln!("에러: {}", e);
    }

    // 비디오 정보만 가져오기
    match example::read_video::get_video_info("video.mp4") {
        Ok(info) => println!("비디오 정보: {:?}", info),
        Err(e) => eprintln!("에러: {}", e),
    }
}
