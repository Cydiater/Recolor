import React, { ChangeEvent, useRef, useState, useEffect } from 'react';
import init, { gen_palette, transfer } from 'recolor';
import { SketchPicker } from 'react-color';
import { lab2rgb, rgb2lab } from './rgblab';

type RGBSqaureInfo = {
    r: number;
    g: number;
    b: number;
};

function App() {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const img = new Image();
    const [palette, setPalette] = useState<RGBSqaureInfo[]>([]);
    const [init_palette, setInitPalette] = useState<RGBSqaureInfo[]>([]);
    const [image_data, setImageData] = useState<Uint8Array>(new Uint8Array());

    const loadImage = async (file: File): Promise<null> => {
        return new Promise((resolve) => {
            const reader = new FileReader();
            reader.onloadend = () => {
                if (typeof reader.result == "string") {
                    img.src = reader.result;
                    resolve(null);
                }
            }
            reader.readAsDataURL(file);
        });
    };

    useEffect(() => {
        console.log("new palette");
        if (init_palette.length === 0) {
            console.log("empty init palette");
            return;
        }
        if (image_data.length === 0) {
            console.log("empty image data");
            return;
        }
        const op = init_palette.map((rgb) => [rgb.r, rgb.g, rgb.b]).flat();
        const np = palette.map((rgb) => [rgb.r, rgb.g, rgb.b]).flat();
        console.log("start transfer");
        const new_data: Uint8Array = transfer(image_data, new Float64Array(op), new Float64Array(np));
        console.log(`width = ${canvasRef.current!.width}`);
        console.log(`height = ${canvasRef.current!.height}`);
        console.log(new_data);
        const img_data = new ImageData(new Uint8ClampedArray(new_data), canvasRef.current!.width);
        canvasRef.current!.getContext('2d')!.putImageData(img_data, 0, 0);
        console.log("done transfer");
    }, [palette]);

    const RGBSquare = (rgb: RGBSqaureInfo, idx: number) => {
        return (
            <div className="flex flex-col m-1" key={idx}>
                <SketchPicker 
                    disableAlpha
                    color={{ r: rgb.r, g: rgb.g, b: rgb.b }}
                    onChangeComplete={(c) => {
                        const np = structuredClone(palette);
                        np[idx].r = c.rgb.r;
                        np[idx].g = c.rgb.g;
                        np[idx].b = c.rgb.b;
                        const lab = rgb2lab([c.rgb.r, c.rgb.g, c.rgb.b]);
                        for (let i = 0; i < idx; i += 1) {
                            const this_lab = rgb2lab([palette[i].r, palette[i].g, palette[i].b]);
                            this_lab[0] = Math.max(this_lab[0], lab[0]);
                            const this_rgb = lab2rgb(this_lab);
                            np[i].r = this_rgb[0];
                            np[i].g = this_rgb[1];
                            np[i].b = this_rgb[2];
                        }
                        for (let i = idx + 1; i < palette.length; i += 1) {
                            const this_lab = rgb2lab([palette[i].r, palette[i].g, palette[i].b]);
                            this_lab[0] = Math.min(this_lab[0], lab[0]);
                            const this_rgb = lab2rgb(this_lab);
                            np[i].r = this_rgb[0];
                            np[i].g = this_rgb[1];
                            np[i].b = this_rgb[2];
                        }
                        setPalette(np);
                    }}
                />
                <div
                    className="m-1 w-8 h-8 border" 
                    style={{
                        backgroundColor: `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`,
                    }}

                />
            </div>)
    }

    return (
        <div className='flex flex-col'> 
            <input type="file" id="recolor-image" accept="image/jpeg, image/png, image/jpg"
                className="flex-none m-1"
                onChange={async (e: ChangeEvent<HTMLInputElement>) => {
                    if (e.target.files != null) {
                        img.onload = () => {
                            const ctx = canvasRef.current!.getContext('2d');
                            canvasRef.current!.height = img.height;
                            canvasRef.current!.width = img.width;
                            ctx?.drawImage(img, 0, 0);
                            canvasRef.current!.toBlob(async (blob) => {
                                const buffer = await blob!.arrayBuffer();
                                const u8array = new Uint8Array(buffer);
                                setImageData(u8array);
                                init().then(() => {
                                    const rgbs = gen_palette(u8array);
                                    const len = rgbs.length;
                                    console.assert(len % 3 === 0);
                                    const palette: RGBSqaureInfo[] = [];
                                    for (let i = 0; i < len; i += 3) {
                                        palette.push({
                                            r: rgbs[i],
                                            g: rgbs[i + 1],
                                            b: rgbs[i + 2],
                                        } as RGBSqaureInfo);
                                    }
                                    setInitPalette(palette);
                                    setPalette(palette);
                                })
                            });
                        }
                        await loadImage(e.target.files[0]);
                    }
                }}
            />
            <div className='flex flex-row'>
                {
                    palette.map((rgb, idx) => RGBSquare(rgb, idx))
                }
            </div>
            <canvas 
                className="flex-none m-1 self-center"
                width={20} height={20} ref={canvasRef} />
        </div>
    );
}

export default App;
