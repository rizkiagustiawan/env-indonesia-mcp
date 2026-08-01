#!/usr/bin/env python3
"""Multi-Panel Publication Figure Generator
Creates journal-quality composite figures from multiple analysis outputs.
Style: Nature/Science/Remote Sensing of Environment standard.
"""
import sys, os, json, argparse
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec
from matplotlib.patches import Rectangle
import numpy as np
from PIL import Image

def create_publication_figure(image_paths, titles, output_path, 
                               main_title="", cols=3, figsize=None,
                               panel_labels=True, dpi=300):
    """Create multi-panel figure from list of image files.
    
    Args:
        image_paths: List of PNG/JPG paths
        titles: List of panel titles (same length as image_paths)
        output_path: Output PNG path
        main_title: Overall figure title
        cols: Number of columns (default 3)
        figsize: Tuple (width, height) in inches. Auto-calculated if None.
        panel_labels: Add (a), (b), (c) labels
        dpi: Output DPI (default 300 for publication)
    """
    n = len(image_paths)
    rows = (n + cols - 1) // cols
    
    if figsize is None:
        figsize = (cols * 5.5, rows * 4.5 + 1.0)
    
    fig = plt.figure(figsize=figsize, dpi=dpi, facecolor='white')
    
    # Main title
    if main_title:
        fig.suptitle(main_title, fontsize=14, fontweight='bold', y=0.98)
    
    gs = gridspec.GridSpec(rows, cols, figure=fig, 
                           hspace=0.25, wspace=0.15,
                           top=0.93, bottom=0.02, left=0.02, right=0.98)
    
    labels = 'abcdefghijklmnopqrstuvwxyz'
    
    for i, (img_path, title) in enumerate(zip(image_paths, titles)):
        row, col = divmod(i, cols)
        ax = fig.add_subplot(gs[row, col])
        
        if os.path.exists(img_path):
            img = Image.open(img_path)
            ax.imshow(np.array(img))
        
        ax.set_xticks([]); ax.set_yticks([])
        for sp in ax.spines.values():
            sp.set_linewidth(0.5)
            sp.set_color('#333333')
        
        # Panel label
        if panel_labels and i < len(labels):
            ax.text(0.02, 0.98, f'({labels[i]})', transform=ax.transAxes,
                    fontsize=11, fontweight='bold', va='top', ha='left',
                    color='white', fontfamily='DejaVu Sans',
                    bbox=dict(boxstyle='square,pad=0.15', fc='#2D3436', ec='none', alpha=0.85))
        
        # Panel title
        ax.set_title(title, fontsize=9, fontweight='bold', pad=4, color='#2D3436')
    
    # Hide empty subplots
    for j in range(n, rows * cols):
        row, col = divmod(j, cols)
        fig.add_subplot(gs[row, col]).set_visible(False)
    
    fig.savefig(output_path, dpi=dpi, bbox_inches='tight', facecolor='white')
    plt.close(fig)
    
    size_kb = os.path.getsize(output_path) / 1024
    print(f"SUCCESS: Publication figure saved: {output_path} ({size_kb:.0f} KB)")
    print(f"Panels: {n} ({rows}x{cols}), DPI: {dpi}")
    return output_path


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Multi-panel publication figure')
    parser.add_argument('--images', nargs='+', required=True, help='List of image paths')
    parser.add_argument('--titles', nargs='+', required=True, help='Panel titles')
    parser.add_argument('--output', required=True, help='Output path')
    parser.add_argument('--main-title', default='', help='Main figure title')
    parser.add_argument('--cols', type=int, default=3, help='Number of columns')
    parser.add_argument('--dpi', type=int, default=300, help='DPI')
    
    args = parser.parse_args()
    create_publication_figure(args.images, args.titles, args.output,
                              main_title=args.main_title, cols=args.cols, dpi=args.dpi)
