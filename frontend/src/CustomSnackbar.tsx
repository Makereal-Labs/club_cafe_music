import {
  Button,
  IconButton,
  Snackbar,
  SnackbarCloseReason,
} from "@mui/material";
import { Close } from "@mui/icons-material";
import React, { useEffect, useState } from "react";

type CustomSnackbarProps = {
  message?: string,
  key: number,
  secondary_action?: string,
  onSecondaryAction?: () => void,
};

type SnackbarMessage = {
  message: string;
  key: number;
};

function CustomSnackbar(props: CustomSnackbarProps) {
  const [snackPack, setSnackPack] = useState<readonly SnackbarMessage[]>([]);
  const [open, setOpen] = useState(false);
  const [messageInfo, setMessageInfo] =
    useState<SnackbarMessage | null>(null);

  useEffect(() => {
    if (props.message) {
      setSnackPack((prev) => [
        ...prev,
        {
          message: props.message!,
          key: props.key,
        },
      ]);
    } else {
      setOpen(false);
    }
  }, [props.message, props.key]);

  useEffect(() => {
    if (snackPack.length > 0) {
      if (messageInfo) {
        // Close an active snack when a new one is added
        setMessageInfo(null);
        setOpen(false);
      } else {
        // Set a new snack when we don't have an active one
        setMessageInfo({ ...snackPack[0] });
        setSnackPack((prev) => prev.slice(1));
        setOpen(true);
      }
    }
  }, [snackPack, messageInfo, open]);

  const handleClose = (
    _event: React.SyntheticEvent | Event,
    reason?: SnackbarCloseReason,
  ) => {
    if (reason === 'clickaway') {
      return;
    }
    setOpen(false);
  };

  return (
    <Snackbar
      key={messageInfo ? messageInfo.key : undefined}
      open={open}
      autoHideDuration={6000}
      onClose={handleClose}
      message={messageInfo ? messageInfo.message : undefined}
      action={
        <>
          {
            props.secondary_action ?
              <Button color="secondary" size="small" onClick={e => {
                handleClose(e);
                if (props.onSecondaryAction) {
                  props.onSecondaryAction();
                }
              }}>
                {props.secondary_action}
              </Button> :
              null
          }
          <IconButton
            aria-label="close"
            color="inherit"
            sx={{ p: 0.5 }}
            onClick={handleClose}
          >
            <Close />
          </IconButton>
        </>}
    />
  );
}

export default CustomSnackbar;