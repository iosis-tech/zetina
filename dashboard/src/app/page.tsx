"use client";

import Button from "@mui/material/Button";
import LinearProgress from "@mui/material/LinearProgress";
import Stepper from "@mui/material/Stepper";
import Step from "@mui/material/Step";
import StepLabel from "@mui/material/StepLabel";
import { DropEvent, FileRejection, useDropzone } from "react-dropzone";
import { ThemeProvider, createTheme, styled } from "@mui/material/styles";
import { StepIconProps } from "@mui/material/StepIcon";
import Check from "@mui/icons-material/Check";
import StepConnector, {
  stepConnectorClasses,
} from "@mui/material/StepConnector";
import { DelegateRequest, DelegateResponse, JobEventsResponse, JobHash, Proof } from "./api";
import { useState } from "react";
import subscribeEvents from "./subscribeEvents";
import assert from "assert";

const steps = [
  "Job propagated to network",
  "Job picked by executor",
  "JobWitness received",
];

const QontoConnector = styled(StepConnector)(({ theme }) => ({
  [`&.${stepConnectorClasses.alternativeLabel}`]: {
    top: 10,
    left: "calc(-50% + 16px)",
    right: "calc(50% + 16px)",
  },
  [`&.${stepConnectorClasses.completed}`]: {
    [`& .${stepConnectorClasses.line}`]: {
      borderColor: "#784af4",
    },
  },
  [`& .${stepConnectorClasses.line}`]: {
    borderColor:
      theme.palette.mode === "dark" ? theme.palette.grey[800] : "#eaeaf0",
    borderTopWidth: 3,
    borderRadius: 1,
  },
}));

const QontoStepIconRoot = styled("div")<{ ownerState: { active?: boolean } }>(
  ({ theme, ownerState }) => ({
    color: theme.palette.mode === "dark" ? theme.palette.grey[700] : "#eaeaf0",
    display: "flex",
    height: 22,
    alignItems: "center",
    "& .QontoStepIcon-completedIcon": {
      color: "#784af4",
      zIndex: 1,
      fontSize: 18,
    },
    "& .QontoStepIcon-circle": {
      width: 8,
      height: 8,
      borderRadius: "50%",
      backgroundColor: "currentColor",
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
      mode: "dark",
      primary: { main: "#784af4", dark: "#784af4" }
    },
  });

  const ondrop = <T extends File>(
    acceptedFiles: T[],
    _fileRejections: FileRejection[],
    _event: DropEvent,
  ) => {
    const file = acceptedFiles[0];
    const reader = new FileReader();

    reader.onload = async (e) => {
      if (e.target && e.target.result) {
        const fileBytes = new Uint8Array(e.target.result as ArrayBuffer);
        console.log(Array.from(fileBytes));
        const requestBody: DelegateRequest = DelegateRequest.parse({
          trace: Array.from(fileBytes),
        });

        console.log(requestBody);

        let subscriber: EventSource | null = null

        try {
          const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL}/delegate`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify(requestBody),
          });

          if (!response.ok) {
            throw new Error(`Error: ${response.statusText}`);
          }

          const data: DelegateResponse = DelegateResponse.parse(
            await response.json(),
          );
          console.log("Job hash:", data.job_hash);

          setActiveStep(1)
          setIsProcessing(data.job_hash);

          subscriber = subscribeEvents(
            `${process.env.NEXT_PUBLIC_API_URL}/job_events`,
            `job_hash=${data.job_hash.toString()}`,
            (event) => {
              let job_event = JobEventsResponse.parse(event);
              if (job_event.type == "Picked") {
                let job_hash = JobHash.parse(job_event.data);
                assert(job_hash == data.job_hash)
                setActiveStep(2)
              }
              if (job_event.type == "Witness") {
                let proof = Proof.parse(job_event.data);
                setActiveStep(3)
                setDownloadBlob([new Blob([new Uint8Array(proof)]), `${data.job_hash}_proof.json`])
                setIsProcessing(null)
                subscriber?.close()
              }
            },
          );
        } catch (error) {
          console.error("Failed to upload file:", error);
          setIsProcessing(null);
          subscriber?.close()
        }
      }
    };

    reader.readAsArrayBuffer(file);
  };

  const [isProcessing, setIsProcessing] = useState<bigint | null>(null);
  const [activeStep, setActiveStep] = useState<number>(0);
  const [downloadBlob, setDownloadBlob] = useState<[Blob, string] | null>(null);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop: ondrop,
  });

  return (
    <ThemeProvider theme={darkTheme}>
      <div className="h-screen flex flex-col background">
        <main className="flex-1 grid justify-center items-center">
          <div className="p-10 border-2 border-gray-800 rounded-2xl backdrop-blur-md grid grid-flow-row gap-8 w-[800px]">
            <div className="text-center font-medium grid grid-flow-row gap-1">
              <div className="text-xl font-bold">ZK prover network</div>
              <div className="text-md">Prove program Trace</div>
            </div>
            <div
              className="cursor-pointer p-10 border-2 rounded-2xl border-dashed border-gray-800 hover:bg"
              {...getRootProps()}
            >
              <input className="w-full" {...getInputProps()} />
              {isProcessing != null ? (
                <p className="text-center">
                  Processing job: {isProcessing.toString()}
                </p>
              ) : isDragActive ? (
                <p className="text-center">Drop the Trace here ...</p>
              ) : (
                <p className="text-center">
                  Drag Trace here, or click to select files
                </p>
              )}
            </div>
            <LinearProgress
              sx={{ backgroundColor: 'transparent', display: isProcessing != null ? 'block' : 'none' }}
            />
            <Stepper
              activeStep={activeStep}
              alternativeLabel
              connector={<QontoConnector />}
            >
              {steps.map((label) => (
                <Step key={label}>
                  <StepLabel StepIconComponent={QontoStepIcon}>
                    {label}
                  </StepLabel>
                </Step>
              ))}
            </Stepper>
            <div className="grid justify-center items-center">
              <Button variant="outlined" size="large" disabled={downloadBlob == null} onClick={() => {
                if (downloadBlob != null) {
                  const download_url = window.URL.createObjectURL(downloadBlob[0]);
                  const a = document.createElement('a');
                  a.href = download_url;
                  a.download = downloadBlob[1];
                  document.body.appendChild(a);
                  a.click();
                  document.body.removeChild(a);
                  window.URL.revokeObjectURL(download_url);
                }
              }}>
                Download Proof
              </Button>
            </div>
          </div>
        </main>
      </div>
    </ThemeProvider>
  );
}
