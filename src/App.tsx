import React, { ChangeEvent, useRef, useState } from 'react';
import init, { gen_palette } from 'recolor';
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
    const [transfering, setTransfering] = useState(false);

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

    const setImage = (data: Uint8Array) => {
        const img_data = new ImageData(new Uint8ClampedArray(data), canvasRef.current!.width);
        canvasRef.current!.getContext('2d')!.putImageData(img_data, 0, 0);
    };

    const Loading = () => {
        return (<div role="status">
            <svg aria-hidden="true" className="mr-2 w-8 h-8 text-gray-200 animate-spin fill-blue-600" viewBox="0 0 100 101" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z" fill="currentColor"/>
                <path d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z" fill="currentFill"/>
            </svg>
            <span className="sr-only">Loading...</span>
        </div>)
    }

    const RGBSquare = (rgb: RGBSqaureInfo, idx: number) => {
        return (
            <div className="flex flex-col m-1" key={idx}>
                <SketchPicker 
                    disableAlpha
                    color={{ r: rgb.r, g: rgb.g, b: rgb.b }}
                    onChangeComplete={(c) => {
                        const np: RGBSqaureInfo[] = structuredClone(palette);
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
                        const op = init_palette.map((rgb) => [rgb.r, rgb.g, rgb.b]).flat();
                        const nnp = np.map((rgb) => [rgb.r, rgb.g, rgb.b]).flat();
                        setTransfering(true);
                        const transferWorker = new Worker(new URL("transfer.js", import.meta.url));
                        transferWorker.postMessage([image_data, new Float64Array(op), new Float64Array(nnp)]);
                        transferWorker.onmessage = (e) => {
                            setImage(e.data);
                            setTransfering(false);
                        };
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
            <div className="flex flex-row">
                <input type="file" id="recolor-image" accept="image/jpeg, image/png, image/jpg"
                    className="flex-none m-1"
                    onChange={async (e: ChangeEvent<HTMLInputElement>) => {
                        if (e.target.files != null) {
                            img.onload = () => {
                                const ctx = canvasRef.current!.getContext('2d');
                                canvasRef.current!.height = img.height;
                                canvasRef.current!.width = img.width;
                                canvasRef.current!.parentElement!.style.width = img.width.toString() + "px";
                                canvasRef.current!.parentElement!.style.height = img.height.toString() + "px";
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
        <button 
            onClick={() => {
                setPalette(init_palette);
                img.onload = () => {
                    const ctx = canvasRef.current!.getContext('2d');
                    canvasRef.current!.height = img.height;
                    canvasRef.current!.width = img.width;
                    ctx?.drawImage(img, 0, 0);
                }
                img.src = URL.createObjectURL(
                    new Blob([image_data.buffer], { type: 'image/png' })
                );
            }}>
                Reset
            </button>
        </div>
        <div className='flex flex-row'>
            {
                palette.map((rgb, idx) => RGBSquare(rgb, idx))
            }
        </div>
        <div className="flex-none m-1 relative">
            <canvas className="absolute" 
                width={20} height={20} ref={canvasRef} >
            </canvas>
            {
                transfering && (
                    <div className="w-full h-full flex content-center justify-center bg-gray-500 bg-opacity-60 z-10 absolute">
                        <div className="absolute z-10 white self-center">
                            {Loading()}
                        </div>
                    </div>)
            }
        </div>
    </div>
);
}

export default App;
