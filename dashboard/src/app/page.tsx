"use client";

import Button from '@mui/material/Button';
import LinearProgress from '@mui/material/LinearProgress';
import Stepper from '@mui/material/Stepper';
import Step from '@mui/material/Step';
import StepLabel from '@mui/material/StepLabel';
import { DropEvent, FileRejection, useDropzone } from 'react-dropzone'
import { ThemeProvider, createTheme, styled } from '@mui/material/styles';
import { StepIconProps } from '@mui/material/StepIcon';
import Check from '@mui/icons-material/Check';
import StepConnector, { stepConnectorClasses } from '@mui/material/StepConnector';

const steps = [
  'Job propagated to network',
  'Job picked by executor',
  'JobWitness received',
];

const QontoConnector = styled(StepConnector)(({ theme }) => ({
  [`&.${stepConnectorClasses.alternativeLabel}`]: {
    top: 10,
    left: 'calc(-50% + 16px)',
    right: 'calc(50% + 16px)',
  },
  [`&.${stepConnectorClasses.active}`]: {
    [`& .${stepConnectorClasses.line}`]: {
      borderColor: '#784af4',
    },
  },
  [`&.${stepConnectorClasses.completed}`]: {
    [`& .${stepConnectorClasses.line}`]: {
      borderColor: '#784af4',
    },
  },
  [`& .${stepConnectorClasses.line}`]: {
    borderColor: theme.palette.mode === 'dark' ? theme.palette.grey[800] : '#eaeaf0',
    borderTopWidth: 3,
    borderRadius: 1,
  },
}));

const QontoStepIconRoot = styled('div')<{ ownerState: { active?: boolean } }>(
  ({ theme, ownerState }) => ({
    color: theme.palette.mode === 'dark' ? theme.palette.grey[700] : '#eaeaf0',
    display: 'flex',
    height: 22,
    alignItems: 'center',
    ...(ownerState.active && {
      color: '#784af4',
    }),
    '& .QontoStepIcon-completedIcon': {
      color: '#784af4',
      zIndex: 1,
      fontSize: 18,
    },
    '& .QontoStepIcon-circle': {
      width: 8,
      height: 8,
      borderRadius: '50%',
      backgroundColor: 'currentColor',
    },
  }),
);

function QontoStepIcon(props: StepIconProps) {
  const { active, completed, className } = props;

  return (
    <QontoStepIconRoot ownerState={{ active }} className={className}>
      {completed ? (
        <Check className="QontoStepIcon-completedIcon" />
      ) : (
        <div className="QontoStepIcon-circle" />
      )}
    </QontoStepIconRoot>
  );
}

export default function Home() {

  const darkTheme = createTheme({
    palette: {
      mode: 'dark',
    },
  });

  const ondrop = <T extends File>(acceptedFiles: T[], fileRejections: FileRejection[], event: DropEvent) => {
    console.log(acceptedFiles)
  }

  const { getRootProps, getInputProps, isDragActive } = useDropzone({ onDrop: ondrop })

  return (
    <ThemeProvider theme={darkTheme}>
      <div className="h-screen flex flex-col background">
        <main className="flex-1 grid justify-center items-center">
          <div className="p-10 border-2 border-gray-800 rounded-2xl backdrop-blur-md grid grid-flow-row gap-8 w-[800px]">
            <div className='text-center font-medium text-xl'>Supply program Trace</div>
            <div className='cursor-pointer p-10 border-2 rounded-2xl border-dashed border-gray-800 hover:bg' {...getRootProps()}>
              <input className='w-full' {...getInputProps()} />
              {
                isDragActive ?
                  <p className='text-center'>Drop the Trace here ...</p> :
                  <p className='text-center'>Drag Trace here, or click to select files</p>
              }
            </div>
            <LinearProgress sx={{ backgroundColor: "transparent" }} />
            <Stepper activeStep={4} alternativeLabel connector={<QontoConnector />}>
              {steps.map((label) => (
                <Step key={label}>
                  <StepLabel StepIconComponent={QontoStepIcon} >{label}</StepLabel>
                </Step>
              ))}
            </Stepper>
            <div className='grid justify-center items-center'>
              <Button variant="outlined" size='large' disabled>
                Download Proof
              </Button>
            </div>
          </div>
        </main>
      </div>
    </ThemeProvider>
  );
}
