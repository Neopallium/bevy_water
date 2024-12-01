# HDR source

From: <https://polyhaven.com/a/table_mountain_2_puresky>
File: table_mountain_2_puresky_4k.hdr
License: CC0

## Conversion to cubemap

Online HDRI to Cubemap: <https://matheowis.github.io/HDRI-to-CubeMap/>
Save output to: 512 piece resolution, PNG, split file layout.

ImageMagick:

```bash
unzip Standard-Cube-Map.zip
convert px.png nx.png py.png ny.png pz.png nz.png -gravity center -append table_mountain_2_puresky_4k_cubemap.jpg
```
