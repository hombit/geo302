use crate::geo::GeoError;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Continent {
    Africa,
    Asia,
    Europe,
    NorthAmerica,
    Oceania,
    SouthAmerica,
    Antarctica,
    Default,
}

impl<'a> TryFrom<&'a str> for Continent {
    type Error = GeoError;

    fn try_from(s: &'a str) -> Result<Self, GeoError> {
        // Upper case is what we use in config
        // Lower case is what is used in ripe-geo
        match s.trim() {
            "Africa" => Ok(Self::Africa),
            "africa" => Ok(Self::Africa),
            "Asia" => Ok(Self::Asia),
            "asia" => Ok(Self::Asia),
            "Europe" => Ok(Self::Europe),
            "europe" => Ok(Self::Europe),
            "NorthAmerica" => Ok(Self::NorthAmerica),
            "north-america" => Ok(Self::NorthAmerica),
            "Oceania" => Ok(Self::Oceania),
            "oceania" => Ok(Self::Oceania),
            "SouthAmerica" => Ok(Self::SouthAmerica),
            "south-america" => Ok(Self::SouthAmerica),
            "Antarctica" => Ok(Self::Antarctica),
            "antarctica" => Ok(Self::Antarctica),
            "default" => Ok(Self::Default),
            _ => Err(GeoError::ContinentUnknown),
        }
    }
}

impl From<Continent> for &'static str {
    fn from(continent: Continent) -> Self {
        match continent {
            Continent::Africa => "Africa",
            Continent::Asia => "Asia",
            Continent::Europe => "Europe",
            Continent::NorthAmerica => "North America",
            Continent::Oceania => "Oceania",
            Continent::SouthAmerica => "South America",
            Continent::Antarctica => "Antarctica",
            Continent::Default => "default",
        }
    }
}
