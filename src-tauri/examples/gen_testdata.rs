// 生成测试素材：不同宽高比、透明图、中文名、子目录、损坏文件
use image::{ImageBuffer, Rgb, RgbImage, Rgba, RgbaImage};
use std::fs;
use std::path::Path;

fn rgb(w: u32, h: u32, c: [u8; 3], label: u32) -> RgbImage {
    let mut img: RgbImage = ImageBuffer::from_pixel(w, h, Rgb(c));
    // 挖个十字/标记，方便肉眼确认没有被拉伸变形
    for i in 0..w.min(h) / 2 {
        let mark = Rgb([255 - c[0], 255 - c[1], 255 - c[2]]);
        img.put_pixel(w / 2, h / 4 + i / 2, mark);
        img.put_pixel(w / 4 + i / 2, h / 2, mark);
    }
    for x in 0..w {
        let v = (x * 255 / w.max(1)) as u8;
        img.put_pixel(x, 0, Rgb([v, label.min(255) as u8, 40]));
    }
    img
}

fn rgba(w: u32, h: u32) -> RgbaImage {
    let mut img: RgbaImage = ImageBuffer::from_pixel(w, h, Rgba([0, 0, 0, 0]));
    for y in 0..h / 2 {
        for x in 0..w / 2 {
            img.put_pixel(x, y, Rgba([255, 40, 40, 255]));
        }
    }
    img
}

fn main() {
    let root = Path::new("testdata/src");
    let _ = fs::remove_dir_all("testdata");
    fs::create_dir_all(root.join("子目录A/深层")).unwrap();
    fs::create_dir_all(root.join("sub-b")).unwrap();

    let colors = [
        ([220, 80, 60], 10),
        ([60, 160, 220], 20),
        ([90, 200, 120], 30),
        ([240, 200, 60], 40),
    ];

    // 横向大图
    for i in 0..10 {
        let (c, l) = colors[i % colors.len()];
        rgb(1600, 900, c, l + i as u32).save(root.join(format!("横幅_{:02}.jpg", i))).unwrap();
    }
    // 竖向大图
    for i in 0..10 {
        let (c, l) = colors[i % colors.len()];
        rgb(800, 1400, c, l + i as u32).save(root.join(format!("竖图-{:02}.png", i))).unwrap();
    }
    // 小图（需要填充）
    for i in 0..6 {
        let (c, l) = colors[i % colors.len()];
        rgb(120, 90, c, l + i as u32).save(root.join("sub-b").join(format!("小图{}.png", i))).unwrap();
    }
    // 透明 PNG
    rgba(900, 900).save(root.join("子目录A/透明素材.png")).unwrap();
    // 深层子目录
    rgb(2000, 600, [130, 60, 200], 70)
        .save(root.join("子目录A/深层/超宽图.png"))
        .unwrap();
    // 同名不同目录（验证目录层级保留）
    rgbb(400, 400).save(root.join("子目录A/同名.png")).unwrap();
    rgbb(400, 400).save(root.join("sub-b/同名.png")).unwrap();
    // 损坏文件
    fs::write(root.join("损坏图片.jpg"), b"this is not a real image").unwrap();
    fs::write(root.join("子目录A/损坏.png"), b"\x89PNG\r\n\x1a\nbroken").unwrap();
    // 不支持格式（应被忽略）
    fs::write(root.join("readme.txt"), b"ignore me").unwrap();

    println!("test data created at {}", root.display());
}

fn rgbb(w: u32, h: u32) -> RgbImage {
    rgb(w, h, [30, 30, 60], 99)
}
