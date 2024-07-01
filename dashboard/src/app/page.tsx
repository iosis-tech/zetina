"use client";
import Button from '@mui/material/Button';
import LinearProgress from '@mui/material/LinearProgress';
import { DropEvent, FileRejection, useDropzone } from 'react-dropzone'


export default function Home() {

  const ondrop = <T extends File>(acceptedFiles: T[], fileRejections: FileRejection[], event: DropEvent) => {
    console.log(acceptedFiles)
  }

  const { getRootProps, getInputProps, isDragActive } = useDropzone({ onDrop: ondrop })

  return (
    <div className="h-screen flex flex-col background">
      <main className="flex-1 grid justify-center items-center">
        <div className="p-10 border-2 border-gray-800 rounded-2xl backdrop-blur-md grid grid-flow-row gap-16 w-[600px]">
          <div className='text-center font-medium text-xl'>Supply program Trace</div>
          <div className='cursor-pointer p-10 border-2 rounded-2xl border-dashed border-gray-800 hover:bg' {...getRootProps()}>
            <input className='w-full' {...getInputProps()} />
            {
              isDragActive ?
                <p className='text-center'>Drop the Trace here ...</p> :
                <p className='text-center'>Drag 'n' drop Trace here, or click to select files</p>
            }
          </div>
          <LinearProgress sx={{ backgroundColor: "transparent" }} />
          <div className='grid justify-center items-center'>
            <Button variant="outlined" size='large'>
              Download Proof
            </Button>
          </div>
        </div>
      </main>
    </div>
  );
}
