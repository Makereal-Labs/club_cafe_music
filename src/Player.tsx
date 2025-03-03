import {
  FastForwardRounded,
  FastRewindRounded,
  PauseRounded,
  PlayArrowRounded,
  VolumeDownRounded,
  VolumeUpRounded,
} from "@mui/icons-material";
import { Box, IconButton, Slider, Stack, Typography, useTheme } from "@mui/material";
import { useState } from "react";

function Player() {
  const theme = useTheme();
  const [playing, setPlaying] = useState(false)

  const current_time = "1:23"
  const remaining_time = "4:56"

  return (
    <Box sx={{
      maxWidth: 350,
      margin: "auto",
      p: 2,
      borderRadius: "4px",
      backgroundColor: theme.palette.background.paper,
    }}>
      <Slider size="small" sx={{ color: "white" }} />
      <Box display="flex" justifyContent="space-between">
        <Typography variant="overline">{current_time}</Typography>
        <Typography variant="overline">-{remaining_time}</Typography>
      </Box>
      <Box sx={{ width: "fit-content", margin: "auto" }}>
        <IconButton>
          <FastRewindRounded fontSize="large" />
        </IconButton>
        <IconButton onClick={() => { setPlaying(!playing) }}>
          {
            playing ? (
              <PlayArrowRounded fontSize="large" />
            ) : (
              <PauseRounded fontSize="large" />
            )
          }
        </IconButton>
        <IconButton>
          <FastForwardRounded fontSize="large" />
        </IconButton>
      </Box>
      <Stack direction="row" alignItems="center" spacing="16px">
        <VolumeDownRounded color="disabled" />
        <Slider />
        <VolumeUpRounded color="disabled" />
      </Stack>
    </Box>
  )
}

export default Player