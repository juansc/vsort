use criterion::{Criterion, criterion_group, criterion_main};
use rand::thread_rng;
use rand::seq::SliceRandom;

fn bench_version_sort() {
    let expected = vec![
        "",
        ".",
        "..",
        ".0",
        ".9",
        ".A",
        ".Z",
        ".a~",
        ".a",
        ".b~",
        ".b",
        ".z",
        ".zz~",
        ".zz",
        ".zz.~1~",
        ".zz.0",
        ".\u{1}",
        ".\u{1}.txt",
        ".\u{1}x",
        ".\u{1}x\u{1}",
        ".\u{1}.0",
        "0",
        "9",
        "A",
        "Z",
        "a~",
        "a",
        "a.b~",
        "a.b",
        "a.bc~",
        "a.bc",
        "a+",
        "a.",
        "a..a",
        "a.+",
        "b~",
        "b",
        "gcc-c++-10.fc9.tar.gz",
        "gcc-c++-10.fc9.tar.gz.~1~",
        "gcc-c++-10.fc9.tar.gz.~2~",
        "gcc-c++-10.8.12-0.7rc2.fc9.tar.bz2",
        "gcc-c++-10.8.12-0.7rc2.fc9.tar.bz2.~1~",
        "glibc-2-0.1.beta1.fc10.rpm",
        "glibc-common-5-0.2.beta2.fc9.ebuild",
        "glibc-common-5-0.2b.deb",
        "glibc-common-11b.ebuild",
        "glibc-common-11-0.6rc2.ebuild",
        "libstdc++-0.5.8.11-0.7rc2.fc10.tar.gz",
        "libstdc++-4a.fc8.tar.gz",
        "libstdc++-4.10.4.20040204svn.rpm",
        "libstdc++-devel-3.fc8.ebuild",
        "libstdc++-devel-3a.fc9.tar.gz",
        "libstdc++-devel-8.fc8.deb",
        "libstdc++-devel-8.6.2-0.4b.fc8",
        "nss_ldap-1-0.2b.fc9.tar.bz2",
        "nss_ldap-1-0.6rc2.fc8.tar.gz",
        "nss_ldap-1.0-0.1a.tar.gz",
        "nss_ldap-10beta1.fc8.tar.gz",
        "nss_ldap-10.11.8.6.20040204cvs.fc10.ebuild",
        "z",
        "zz~",
        "zz",
        "zz.~1~",
        "zz.0",
        "zz.0.txt",
        "\u{1}",
        "\u{1}.txt",
        "\u{1}x",
        "\u{1}x\u{1}",
        "\u{1}.0",
        "#\u{1}.b#",
        "#.b#",
    ];
    let mut list = expected.clone();
    list.shuffle(&mut thread_rng());
    vsort::sort(&mut list);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("version sort", |b| b.iter(|| bench_version_sort()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);