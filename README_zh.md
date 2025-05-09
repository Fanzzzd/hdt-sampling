# HDT Sampling (分层飞镖投掷)

[![CI](https://github.com/Fanzzzd/hdt_sampling/actions/workflows/CI.yml/badge.svg)](https://github.com/Fanzzzd/hdt_sampling/actions/workflows/CI.yml)
[![PyPI](https://img.shields.io/pypi/v/hdt-sampling?logo=pypi)](https://pypi.org/project/hdt-sampling/)

[English Readme (英文说明)](README.md)

一个使用 Rust 实现的快速 Python 库，用于通过分层飞镖投掷 (Hierarchical Dart Throwing, HDT) 算法生成二维泊松圆盘 (Poisson Disk) 点集。泊松圆盘采样生成的点紧密排列，但任意两点之间的距离都不小于指定的最小距离，从而产生高质量的蓝噪声分布。

该实现基于以下论文：

> K. B. White, D. Cline and P. K. Egbert, "Poisson Disk Point Sets by Hierarchical Dart Throwing," *2007 IEEE Symposium on Interactive Ray Tracing*, Ulm, Germany, 2007, pp. 129-132, doi: 10.1109/RT.2007.4342600.
> ([论文链接 BYU ScholarsArchive](https://scholarsarchive.byu.edu/facpub/237))

## 特性

*   在矩形域内生成二维泊松圆盘样本点。
*   使用 Rust 实现，速度快，可从 Python 调用。
*   生成最大点集（在满足最小距离规则的前提下，尽可能填满空间）。
*   简洁的 API。

## 性能

在 Apple M3 Max 芯片上，对于 10000x10000 的区域和 20.0 的最小距离，生成约 170,000 个点大约需要 **0.78 秒**。

## 安装

你可以通过 PyPI 安装：

```bash
pip install hdt-sampling
```

或者，如果你安装了 Rust 和 Maturin，可以从源码构建：

```bash
# 克隆仓库
git clone https://github.com/Fanzzzd/hdt_sampling.git
cd hdt_sampling

# 安装 maturin (如果尚未安装)
# pip install maturin

# 在当前 Python 环境中构建并安装
maturin develop --release
# 或者构建 wheel 文件用于分发
# maturin build --release
```

## 使用方法

```python
import time
import matplotlib.pyplot as plt
from hdt_sampling import HDTSampler

# --- 参数 ---
width = 1000.0  # 区域宽度
height = 1000.0 # 区域高度
min_dist = 15.0 # 点之间的最小距离

# --- 生成点 ---
print("初始化 HDT 采样器...")
start_init = time.time()
sampler = HDTSampler(width, height, min_dist)
end_init = time.time()
print(f"初始化耗时: {end_init - start_init:.4f} 秒")

print("生成点...")
start_gen = time.time()
points = sampler.generate() # 返回一个包含 (x, y) 元组的列表
end_gen = time.time()
print(f"生成耗时: {end_gen - start_gen:.4f} 秒")
print(f"生成了 {len(points)} 个点。")

# --- 绘图 ---
if points:
    x_coords, y_coords = zip(*points)

    plt.figure(figsize=(8, 8))
    plt.scatter(x_coords, y_coords, s=5, c='blue', alpha=0.7)
    plt.xlim(0, width)
    plt.ylim(0, height)
    plt.xlabel("X")
    plt.ylabel("Y")
    plt.title(f"泊松圆盘点 (r={min_dist}) - {len(points)} 点")
    plt.gca().set_aspect('equal', adjustable='box')
    plt.grid(True, linestyle='--', alpha=0.4)
    plt.show()
else:
    print("没有生成任何点。")
```

查看 `example/example.ipynb` 文件获取更详细的示例，包括傅里叶分析绘图。

## 许可证

本项目使用 MIT 许可证。详情请参阅 LICENSE 文件。