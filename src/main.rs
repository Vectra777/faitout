use  iced::{widget::row,
            Element};


fn main() -> iced::Result {
    iced::run("faitout", App::update, App::view)
}


#[derive(Default,Debug, Clone)]
struct Message;


#[derive(Default)]
struct App;

struct Editor{

}

impl App{
    fn update(&mut self, message: Message){
    }


    fn view(&self) -> Element<Message>{
        row![].into()

    }

    fn theme(&self) -> iced::Theme{
        iced::Theme::SolarizedDark
    }
}
