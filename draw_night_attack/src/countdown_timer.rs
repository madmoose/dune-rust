use num::traits::WrappingSub;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountdownTimer<T> {
    name: String,
    limit: T,
    value: T,
}

impl<T> CountdownTimer<T>
where
    T: Copy + PartialOrd + WrappingSub + num::One + std::fmt::LowerHex,
{
    pub fn new(name: &str, value: T, limit: T) -> Self {
        CountdownTimer {
            name: name.to_owned(),
            value,
            limit,
        }
    }

    pub fn set(&mut self, value: T) {
        println!("CountdownTimer {} set:  {:#x}", self.name, value);
        self.value = value;
    }

    pub fn value(&self) -> T {
        self.value
    }

    pub fn set_value(&mut self, value: T) {
        println!("CountdownTimer {} set_value:  {:#x}", self.name, value);
        self.value = value;
    }

    fn triggered(&self) -> bool {
        self.value < self.limit
    }

    pub fn tick(&mut self) -> bool {
        let new_value = self.value.wrapping_sub(&T::one());

        println!(
            "CountdownTimer {} tick: {:#x} -> {:#x}{}",
            self.name,
            self.value,
            new_value,
            if new_value < self.limit {
                " - triggered"
            } else {
                ""
            },
        );

        self.value = new_value;
        self.triggered()
    }
}
