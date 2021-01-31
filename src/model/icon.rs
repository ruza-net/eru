use std::path::PathBuf;

use crate::behavior::View;

use iced::{
    image,
    Color,
};



#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    color: [f32; 4],
    text: Option<String>,
}

enum Data {
    Image(image::Handle),
    Text(String),

    Element(Box<dyn View>),
}



pub struct Icon {
    data: Data,

    color: Color,
}


impl Icon {
    pub fn from_file(res_path: PathBuf) -> Self {
        let mut img_path = res_path.clone();
        img_path.push("img.png");

        let mut color_path = res_path;
        color_path.push("config.ron");

        let mut buf = String::new();

        file_contents!{ color_path >> buf }

        let Config { mut color, text } = ron::from_str(&buf).expect("invalid data in `color.ron`");

        color[0] /= 256.;
        color[1] /= 256.;
        color[2] /= 256.;

        let data = if let Some(text) = text {
            Data::Text(text)

        } else {
            Data::Image(image::Handle::from_path(img_path))
        };

        Self {
            data,

            color: color.into(),
        }
    }
    pub fn from_text(txt: impl ToString, color: impl Into<Color>) -> Self {
        let data = Data::Text(txt.to_string());

        Self {
            data,

            color: color.into(),
        }
    }
    pub fn from(el: Box<dyn View>, color: impl Into<Color>) -> Self {
        let data = Data::Element(el);

        Self {
            data,

            color: color.into(),
        }
    }

    pub fn view(&self, size: Option<u16>) -> iced::Element<'static, ()> {
        use iced::Length;

        match &self.data {
            Data::Image(img) => iced::Image::new(img.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),

            Data::Element(el) => el.view(),

            Data::Text(txt) => {
                let mut txt = iced::Text::new(txt.clone())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .horizontal_alignment(iced::HorizontalAlignment::Center)
                    .vertical_alignment(iced::VerticalAlignment::Center);

                if let Some(size) = size {
                    txt = txt.size((size as f32 / 1.2) as u16);
                }

                txt.into()
            },
        }
    }
    pub fn color(&self) -> Color {
        self.color
    }
}
