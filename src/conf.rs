use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    actors: Vec<Actor>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Actor {
    Emitter(Emitter),
    Minuterie(Minuterie),
    Outlet(Outlet),
    Switch(Switch),
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Emitter {
    #[serde(with = "humantime_serde")]
    interval: Duration,
    topic: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Minuterie {
    #[serde(with = "humantime_serde")]
    timeout: Duration,
    topic: String,
    input: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Outlet {
    control_topic: String,
    status_topic: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Switch {
    topic: String,
}


#[cfg(test)]
mod tests {
    use figment::Figment;
    use crate::conf::{Actor, Config, Emitter, Minuterie, Outlet, Switch};

    use figment::providers::{Format, Toml};


    #[test]
    fn print_sample_config() {
        let config = Config {
            actors: vec![
                Actor::Emitter(Emitter {
                    interval: Default::default(),
                    topic: "".to_string(),
                }),
                Actor::Emitter(Emitter {
                    interval: Default::default(),
                    topic: "".to_string(),
                }),
                Actor::Minuterie(Minuterie {
                    timeout: Default::default(),
                    topic: "".to_string(),
                    input: vec![],
                }),
                Actor::Outlet(Outlet {
                    control_topic: "".to_string(),
                    status_topic: "".to_string(),
                }),
                Actor::Switch(Switch {
                    topic: "".to_string(),
                })
            ],
        };
        let toml_string = toml::to_string(&config).unwrap();
        println!("{toml_string}");


        let config: Config = Figment::new()
            .merge(Toml::string(include_str!("config.sample.toml")))
            .extract()
            .unwrap();

        println!("{:?}", config);
    }
}