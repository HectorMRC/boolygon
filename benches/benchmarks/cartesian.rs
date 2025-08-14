use boolygon::{cartesian::Polygon, Shape, Tolerance};
use criterion::{criterion_group, BatchSize, Criterion};
use rand::Rng;

type Sample = [[f64; 2]; 1000];

fn random_shape() -> Shape<Polygon<f64>> {
    let mut rng = rand::rng();

    Shape::from(Polygon::from(rng.random::<Sample>().to_vec()))
}

fn random_operands() -> [Shape<Polygon<f64>>; 2] {
    [random_shape(), random_shape()]
}

pub fn large_shapes(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("large shapes");

    group.bench_function("join", |b| {
        b.iter_batched(
            || random_operands(),
            |[subject, clip]| {
                subject.or(clip, Tolerance::default());
            },
            BatchSize::LargeInput,
        )
    });

    group.bench_function("difference", |b| {
        b.iter_batched(
            || random_operands(),
            |[subject, clip]| {
                subject.not(clip, Tolerance::default());
            },
            BatchSize::LargeInput,
        )
    });

    group.bench_function("intersection", |b| {
        b.iter_batched(
            || random_operands(),
            |[subject, clip]| {
                subject.and(clip, Tolerance::default());
            },
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, large_shapes);
