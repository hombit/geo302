#!/usr/bin/env python3

import tomllib
from itertools import product
from typing import Collection, Generator, Iterable, Iterator, Set, TypeVar


NOT_USED_IN_CFG = ["full", "default"]
ONE_OF_IS_REQUIRED = frozenset(["maxminddb", "ripe-geo"])


T = TypeVar("T")


def mask_index(l: Iterable[T], mask: Iterable[bool]) -> Iterator[T]:
    return (x for x, flag in zip(l, mask) if flag)


def combinations(c: Collection[T], full_first: bool = False) -> Generator[Set[T], None, None]:
    n = len(c)
    if full_first:
        flags = [True, False]
    else:
        flags = [False, True]
    for mask in product(flags, repeat=n):
        yield set(mask_index(c, mask))


def main():
    with open("Cargo.toml", "rb") as fh:
        cargo_toml = tomllib.load(fh)
    features = cargo_toml["features"]

    # Remove features that are not used in the #[cfg] statements in the code
    for feature in NOT_USED_IN_CFG:
        del features[feature]

    # filter out Cargo dependencies
    features = {
        feature: frozenset(dep for dep in deps if not dep.startswith("dep:") and "/" not in dep)
        for feature, deps in features.items()
    }

    list_features = list(features)
    combos = set()
    for combo in combinations(list_features, full_first=True):
        if len(ONE_OF_IS_REQUIRED & combo) == 0:
            continue
        # filter out redundant feature pairs, i.e. feature and its dependency
        for feature in combo.copy():
            combo -= features[feature]
        combos.add(frozenset(combo))

    for combo in sorted(combos):
        print(",".join(sorted(combo)))


if __name__ == "__main__":
    main()
