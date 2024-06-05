use anyhow::Result;
use std::io;

pub trait Measurement<T> {
    // T is the type of measurement
    fn get_measurement(&mut self, timestamp: u128) -> T;

    // for matching multiple measurements at a time
    fn get_multiple_measurements(&mut self, timestamps: &Vec<u128>) -> Vec<T>;
}

pub trait Build {
    fn build(&self, repo: String) -> Result<(), io::Error>; // returns whether it succeded
}

pub trait Listener<T> {
    fn start_listening<B: Build, M: Measurement<T>>(
        &self,
        builder: B,
        measurement: &mut M,
    ) -> Result<()>;
}
