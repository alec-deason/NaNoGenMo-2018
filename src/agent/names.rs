use rand::seq::SliceRandom;

static MALE_FIRST_NAMES: &'static [&str] = &["Liam", "Noah", "William", "James", "Logan", "Benjamin", "Mason", "Elijah", "Oliver", "Jacob", "John", "Robert", "Michael", "David", "Richard", "Charles", "Joseph", "Thomas", "Santiago", "Mateo", "Matías", "Diego", "Sebastián", "Nicolás", "Iker", "Alejandro", "Samuel"];

static FEMALE_FIRST_NAMES: &'static [&str] = &["Emma", "Olivia", "Ava", "Isabella", "Sophia", "Mia", "Charlotte", "Amelia", "Evelyn", "Abigail", "Dorothy", "Margaret", "Susan", "Maria", "Jennifer", "Elizabeth", "Barbara", "Linda", "Patricia", "Mary", "Mariana", "Valentina", "Isabella", "Sofía", "Valeria", "Gabriela", "Sara", "Salomé", "Daniela"];

static SURNAMES: &'static [&str] = &["Smith", "Johnson", "Williams", "Brown", "Jones", "Miller", "Davis", "Garcia", "Rodriguez", "Wilson", "Martinez", "Anderson", "Taylor", "Thomas", "Hernandez", "Moore", "Martin", "Jackson", "Thompson", "White", "Lopez", "Lee", "Gonzalez", "Harris", "Clark", "Lewis", "Robinson", "Walker", "Perez", "Hall", "Young",];

pub fn male_name() -> String {
    let mut rng = rand::thread_rng();
    let first_name = MALE_FIRST_NAMES.choose(&mut rng).unwrap();
    let last_name = SURNAMES.choose(&mut rng).unwrap();

    format!("{} {}", first_name, last_name).to_string()
}

pub fn female_name() -> String {
    let mut rng = rand::thread_rng();
    let first_name = FEMALE_FIRST_NAMES.choose(&mut rng).unwrap();
    let last_name = SURNAMES.choose(&mut rng).unwrap();

    format!("{} {}", first_name, last_name).to_string()
}
