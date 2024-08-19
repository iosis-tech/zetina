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
import {
  DelegateRequest,
  DelegateResponse,
  JobEventsResponse,
  PeerId,
  Proof,
} from "./api";
import { useEffect, useRef, useState } from "react";
import subscribeEvents from "./subscribeEvents";
import { WorkerMessage, WorkerResponse } from "@/utils/types";
import { matchCommitment, matchLayout } from "@/utils/loadModule";

const steps = [
  "Job sent",
  "Job propagated",
  "Job bidding",
  "Job delegated",
  "Proof received",
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
  const workerRef = useRef<Worker>();
  const darkTheme = createTheme({
    palette: {
      mode: "dark",
      primary: { main: "#784af4", dark: "#784af4" },
    },
  });

  // Function to add a new log entry
  const addLog = (message: string) => {
    const now = new Date();
    const timestamp = now.toLocaleString(); // Get current date and time as a string
    const logEntry = (
      <div key={logs.length}>
        LOG: {timestamp} - {message}
      </div>
    );
    setLogs((prevLogs) => [...prevLogs, logEntry]);
  };

  const verifyProof = async (proof: string) => {
    const parsedProof = JSON.parse(proof);

    const layout = matchLayout(parsedProof.public_input.layout);
    const commitment = matchCommitment(parsedProof.proof_parameters.pow_hash);

    workerRef.current = new Worker(new URL("../worker.ts", import.meta.url), {
      type: "module",
    });

    workerRef.current.onmessage = (event: MessageEvent<WorkerResponse>) => {
      const { programHash, programOutput, error } = event.data;

      if (error) {
        console.error(error);
        addLog("Verification Failed");
        setButtonColor("error");
      } else {
        addLog(`programHash: ${programHash}`);
        addLog(`programOutput: ${programOutput}`);
        addLog(`layout: ${layout}`);
        addLog(`commitment: ${commitment}`);
        addLog("Proof Verified");
        setButtonColor("success");
      }

      workerRef.current?.terminate();
    };

    if (layout && commitment) {
      const message: WorkerMessage = {
        proof,
        layout,
        commitment,
      };

      workerRef.current.postMessage(message);
    }
  };

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
          pie: Array.from(fileBytes),
        });

        let subscriber: EventSource | null = null;

        try {
          const response = await fetch(
            `${process.env.NEXT_PUBLIC_API_URL}/delegate`,
            {
              method: "POST",
              headers: {
                "Content-Type": "application/json",
              },
              body: JSON.stringify(requestBody),
            },
          );

          if (!response.ok) {
            throw new Error(`Error: ${response.statusText}`);
          }

          const data: DelegateResponse = DelegateResponse.parse(
            await response.json(),
          );
          addLog(`Job ${data.job_key} sent to delegator`);
          setActiveStep(1);
          setIsProcessing(data.job_key);

          subscriber = subscribeEvents(
            `${process.env.NEXT_PUBLIC_API_URL}/job_events`,
            `job_key=${data.job_key.toString()}`,
            async (event) => {
              let job_event = JobEventsResponse.parse(event);
              if (job_event.type == "Propagated") {
                addLog(
                  `Job ${data.job_key} propagated to network DHT and gossip topics`,
                );
                setActiveStep(2);
              }
              if (job_event.type == "BidReceived") {
                let peer_id = PeerId.parse(job_event.data);
                addLog(
                  `Recived bid for job ${data.job_key} from peer ${peer_id}`,
                );
                setActiveStep(3);
              }
              if (job_event.type == "Delegated") {
                let peer_id = PeerId.parse(job_event.data);
                addLog(`Job ${data.job_key} delegated to peer ${peer_id}`);
                setActiveStep(4);
              }
              if (job_event.type == "Finished") {
                let proof = Proof.parse(job_event.data);
                addLog(`Job ${data.job_key} proof received`);
                setDownloadBlob([
                  new Blob([new Uint8Array(proof)]),
                  `${data.job_key}_proof.json`,
                ]);
                setActiveStep(5);
                setIsProcessing(null);
                subscriber?.close();
                await verifyProof(
                  new TextDecoder().decode(new Uint8Array(job_event.data)),
                );
              }
            },
          );
        } catch (error) {
          console.error("Failed to upload file:", error);
          setIsProcessing(null);
          subscriber?.close();
        }
      }
    };

    reader.readAsArrayBuffer(file);
  };

  const [isProcessing, setIsProcessing] = useState<string | null>(null);
  const [logs, setLogs] = useState<JSX.Element[]>([]);
  const [activeStep, setActiveStep] = useState<number>(0);
  const [downloadBlob, setDownloadBlob] = useState<[Blob, string] | null>(null);
  const [buttonColor, setButtonColor] = useState<
    | "inherit"
    | "primary"
    | "secondary"
    | "success"
    | "error"
    | "info"
    | "warning"
  >("primary");

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop: ondrop,
  });

  const scrollContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollContainerRef.current) {
      scrollContainerRef.current.scrollTop =
        scrollContainerRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <ThemeProvider theme={darkTheme}>
      <div className="h-screen flex flex-col">
        <main className="flex-1 grid items-center">
          <div className="p-10 border-2 border-gray-800 rounded-2xl backdrop-blur-md grid grid-flow-row gap-8 max-w-[1000px] w-full mx-auto">
            <div className="text-center font-medium grid grid-flow-row gap-1">
              <div className="text-xl font-bold">Zetina network</div>
              <div className="text-md">Prove program Pie</div>
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
                <p className="text-center">Drop the Pie here ...</p>
              ) : (
                <p className="text-center">
                  Drag Pie here, or click to select files
                </p>
              )}
            </div>
            <LinearProgress
              sx={{
                backgroundColor: "transparent",
                display: isProcessing != null ? "block" : "none",
              }}
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
            <div
              ref={scrollContainerRef}
              className="scroll-container p-1 px-4 border-2 border-gray-800 rounded-2xl backdrop-blur-md h-32 overflow-y-scroll text-xs text-wrap break-words text-gray-500"
            >
              {logs.map((log, index) => (
                <div key={index}>{log}</div>
              ))}
            </div>
            <div className="grid justify-center items-center">
              <Button
                variant="outlined"
                size="large"
                color={buttonColor}
                disabled={downloadBlob == null}
                onClick={() => {
                  if (downloadBlob != null) {
                    const download_url = window.URL.createObjectURL(
                      downloadBlob[0],
                    );
                    const a = document.createElement("a");
                    a.href = download_url;
                    a.download = downloadBlob[1];
                    document.body.appendChild(a);
                    a.click();
                    document.body.removeChild(a);
                    window.URL.revokeObjectURL(download_url);
                  }
                }}
              >
                Download Proof
              </Button>
            </div>
          </div>
        </main>
      </div>
    </ThemeProvider>
  );
}
