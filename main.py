#!/usr/bin/env python3
import argparse
import os
from collections import namedtuple

Node = namedtuple("Node", "module, rank, path, flow, name")


def read_partition(filename):
    header = []
    nodes = {}

    with open(filename) as f:
        lines = f.readlines()

    for line in lines:
        line = line.strip()

        if line.startswith("#"):
            header.append(line)
            continue

        # for first-order networks with node names without whitespaces
        path, flow, name, node = line.split()
        flow = float(flow)
        node = int(node)
        path_arr = path.split(":")
        module = ":".join(path_arr[0:-1])
        rank = path_arr[-1]
        nodes[node] = Node([module], rank, path, flow, name)

    return header, nodes


def read_partitions(partitions):
    first, *rest = partitions
    print(f"Core partition:\n\t{first}")
    print(f"Rest partitions:")
    for partition in rest:
        print(f"\t{partition}")

    header, core = read_partition(first)

    for partition in rest:
        _, other = read_partition(partition)

        for node_id, node in other.items():
            core[node_id].module.append(node.module[0])

    return header, core


def write_aggregated_partitions(aggregated, result_file):
    print(f"Writing aggregated partitions to {result_file}")
    with open(result_file, "w") as f:
        for node_id in sorted(aggregated):
            node = aggregated[node_id]
            paths = " ".join(map(str, node[0]))
            f.write(f"{node_id} {paths}\n")


def run_significance_clustering(agg_file, result_file):
    binary="./target/release/significance-clustering"
    cmd = f"{binary} {agg_file} {result_file}"
    print(f"Running significance-clustering: {cmd}")
    print("----------------------------------------")
    os.system(cmd)
    print("----------------------------------------")


def write_aggregated_tree(header, aggregated, result_file, tree_file):
    print(f"Reading result file {result_file}")
    with open(result_file) as f:
        lines = f.readlines()

    print(f"Writing aggregated tree to {tree_file}")
    with open(tree_file, "w") as f:
        f.write("\n".join(header))
        f.write("\n")

        for line in lines:
            line = line.strip()
            path, node_id = line.split(" ")
            node_id = int(node_id)

            node = aggregated[node_id]

            insignificant = path.endswith(";")
            sep = "" if insignificant else ":"

            path += sep + node.rank

            if ";" in path:
                path += ";"

            f.write(f"{path} {node.flow} {node.name} {node_id}\n")


def main(filenames, agg_file, result_file, tree_file):
    header, aggregated = read_partitions(filenames)
    write_aggregated_partitions(aggregated, agg_file)
    run_significance_clustering(agg_file, result_file)
    write_aggregated_tree(header, aggregated, result_file, tree_file)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Run significance clustering on tree files")
    parser.add_argument("agg_file", help="Aggregated partition file")
    parser.add_argument("tree_file", help="Output tree file")
    parser.add_argument("filenames", nargs="+", help="Input tree files, first one is raw partition")
    args = parser.parse_args()

    agg_file_name, ext = os.path.splitext(args.agg_file)
    result_file = agg_file_name + "_output" + ext

    main(args.filenames, args.agg_file, result_file, args.tree_file)
