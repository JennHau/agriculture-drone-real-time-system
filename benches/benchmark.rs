use std::time::Instant;
use criterion::{Criterion, criterion_group, criterion_main, Throughput};
use agriculture_drone::benchmark_functions::actuators_to_main_control_bm::bm_actuators_to_main_control;
use agriculture_drone::benchmark_functions::battery_management_bm::bm_battery_management;
use agriculture_drone::benchmark_functions::gps_bm::bm_gps;
use agriculture_drone::benchmark_functions::main_control_to_actuators_bm::bm_main_control_to_actuators;
use agriculture_drone::benchmark_functions::monitor_bm::bm_monitor;
use agriculture_drone::benchmark_functions::multispectral_camera_bm::bm_multispectral_camera;
use agriculture_drone::benchmark_functions::sensors_to_monitor_to_main_control_bm::bm_sensors_to_monitor_to_main_control;
use agriculture_drone::benchmark_functions::thermal_bm::bm_thermal_sensor;
use agriculture_drone::benchmark_functions::ultrasonic_sensor_bm::bm_ultrasonic_sensor;

struct LatencyBenchmark {
    start: Option<Instant>,
    latencies: Vec<u128>,
}

impl LatencyBenchmark {
    fn new() -> Self {
        Self {
            start: None,
            latencies: Vec::new(),
        }
    }

    fn op_start(&mut self) {
        self.start = Some(Instant::now());
    }

    fn op_finish(&mut self) {
        if let Some(start_time) = self.start {
            let elapsed = start_time.elapsed().as_nanos();
            self.latencies.push(elapsed);
            self.start = None;
        }
    }

    fn print(&self) {
        if self.latencies.is_empty() {
            return;
        }

        let sum: u128 = self.latencies.iter().sum();
        let avg = sum / self.latencies.len() as u128;
        let min = *self.latencies.iter().min().unwrap();
        let max = *self.latencies.iter().max().unwrap();

        println!("Latency (ns) avg: {}, min: {}, max: {}", avg, min, max);
    }
}



fn bm_multispectral_camera_with_lat(lb: &mut LatencyBenchmark) {
    lb.op_start();
    bm_multispectral_camera();
    lb.op_finish()
}

pub fn criterion_benchmark_multispectral_sensor(c: &mut Criterion) {
    let mut lb = LatencyBenchmark::new();

    let mut group = c.benchmark_group("Sensors");
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("Multispectral Camera", |b|b.iter(
        ||bm_multispectral_camera_with_lat(&mut lb)));

    group.finish();
    lb.print();
}

fn bm_thermal_sensor_with_lat(lb: &mut LatencyBenchmark) {
    lb.op_start();
    bm_thermal_sensor();
    lb.op_finish()
}

pub fn criterion_benchmark_thermal_sensor(c: &mut Criterion) {
    let mut lb = LatencyBenchmark::new();

    let mut group = c.benchmark_group("Sensors");
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("Thermal Sensor", |b|b.iter(
        ||bm_thermal_sensor_with_lat(&mut lb)));

    group.finish();
    lb.print();
}

fn bm_gps_with_lat(lb: &mut LatencyBenchmark) {
    lb.op_start();
    bm_gps();
    lb.op_finish()
}

pub fn criterion_benchmark_gps(c: &mut Criterion) {
    let mut lb = LatencyBenchmark::new();

    let mut group = c.benchmark_group("Sensors");
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("GPS", |b|b.iter(||bm_gps_with_lat(&mut lb)));

    group.finish();
    lb.print();
}

fn bm_ultrasonic_sensor_with_lat(lb: &mut LatencyBenchmark) {
    lb.op_start();
    bm_ultrasonic_sensor();
    lb.op_finish()
}

pub fn criterion_benchmark_ultrasonic_sensor(c: &mut Criterion) {
    let mut lb = LatencyBenchmark::new();

    let mut group = c.benchmark_group("Sensors");
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("Ultrasonic Sensor", |b|b.iter(
        ||bm_ultrasonic_sensor_with_lat(&mut lb)));

    group.finish();
    lb.print();
}

fn bm_battery_management_with_lat(lb: &mut LatencyBenchmark) {
    lb.op_start();
    bm_battery_management();
    lb.op_finish()
}

pub fn criterion_benchmark_battery_management(c: &mut Criterion) {
    let mut lb = LatencyBenchmark::new();

    let mut group = c.benchmark_group("Sensors");
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("Battery Management", |b|b.iter(
        ||bm_battery_management_with_lat(&mut lb)));

    group.finish();
    lb.print();
}

fn bm_monitor_with_lat(lb: &mut LatencyBenchmark) {
    lb.op_start();
    bm_monitor();
    lb.op_finish()
}

pub fn criterion_benchmark_monitor(c: &mut Criterion) {
    let mut lb = LatencyBenchmark::new();

    let mut group = c.benchmark_group("Sensors");
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("Monitor", |b|b.iter(
        ||bm_battery_management_with_lat(&mut lb)));

    group.finish();
    lb.print();
}

fn bm_sensors_to_monitor_to_main_control_with_lat(lb: &mut LatencyBenchmark) {
    lb.op_start();
    bm_sensors_to_monitor_to_main_control();
    lb.op_finish()
}

pub fn criterion_benchmark_sensors_to_monitor_to_main_control(c: &mut Criterion) {
    let mut lb = LatencyBenchmark::new();

    let mut group = c.benchmark_group("Cross Functions");
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("Sensors to Monitor to Main Control", |b|b.iter(
        ||bm_sensors_to_monitor_to_main_control_with_lat(&mut lb)));

    group.finish();
    lb.print();
}

fn bm_main_control_to_actuators_with_lat(lb: &mut LatencyBenchmark) {
    lb.op_start();
    bm_main_control_to_actuators();
    lb.op_finish()
}

pub fn criterion_benchmark_main_control_to_actuators(c: &mut Criterion) {
    let mut lb = LatencyBenchmark::new();

    let mut group = c.benchmark_group("Cross Functions");
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("Main Control to Actuators", |b|b.iter(
        ||bm_main_control_to_actuators_with_lat(&mut lb)));

    group.finish();
    lb.print();
}

fn bm_actuators_to_main_control_with_lat(lb: &mut LatencyBenchmark) {
    lb.op_start();
    bm_actuators_to_main_control();
    lb.op_finish()
}

pub fn criterion_benchmark_actuators_to_main_control(c: &mut Criterion) {
    let mut lb = LatencyBenchmark::new();

    let mut group = c.benchmark_group("Cross Functions");
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("Actuators to Main Control", |b|b.iter(
        ||bm_actuators_to_main_control_with_lat(&mut lb)));

    group.finish();
    lb.print();
}

fn overall_simulation() {
    bm_sensors_to_monitor_to_main_control();
    bm_main_control_to_actuators();
    bm_actuators_to_main_control();
}

fn bm_overall_simulation_with_lat(lb: &mut LatencyBenchmark) {
    lb.op_start();
    overall_simulation();
    lb.op_finish()
}

pub fn criterion_benchmark_overall_simulation(c: &mut Criterion) {
    let mut lb = LatencyBenchmark::new();

    let mut group = c.benchmark_group("Overall Simulation");
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("Overall Simulation", |b|b.iter(
        ||bm_overall_simulation_with_lat(&mut lb)));

    group.finish();
    lb.print();
}

criterion_group!(benches,
    // criterion_benchmark_multispectral_sensor
    // criterion_benchmark_thermal_sensor
    // criterion_benchmark_gps
    // criterion_benchmark_ultrasonic_sensor
    // criterion_benchmark_battery_management
    // criterion_benchmark_monitor
    // criterion_benchmark_sensors_to_monitor_to_main_control
    // criterion_benchmark_main_control_to_actuators
    // criterion_benchmark_actuators_to_main_control
    criterion_benchmark_overall_simulation
);
criterion_main!(benches);