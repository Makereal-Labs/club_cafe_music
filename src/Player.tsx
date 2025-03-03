import {
  FastForwardRounded,
  FastRewindRounded,
  PauseRounded,
  PlayArrowRounded,
  VolumeDownRounded,
  VolumeUpRounded,
} from "@mui/icons-material";
import {
  Box,
  IconButton,
  Slider,
  Stack,
  Typography,
  useTheme,
} from "@mui/material";
import { useState } from "react";

function format_time(time: number): String {
  return `${Math.floor(time / 60)}:` + `${Math.floor(time) % 60}`.padStart(2, "0");
}

function Player() {
  const theme = useTheme();
  const [playing, setPlaying] = useState(false);
  const [play_time, setPlayTime] = useState(0);
  const total_time = 180;

  const current_time = format_time(play_time);
  const remaining_time = format_time(total_time - play_time);

  function on_playtime_slider_change(time: number) {
    setPlayTime(time);
  }

  return (
    <Box sx={{
      maxWidth: 350,
      margin: "auto",
      p: 2,
      borderRadius: "4px",
      backgroundColor: theme.palette.background.paper,
    }}>
      <Slider
        size="small"
        sx={{ color: "white" }}
        min={0}
        step={1}
        max={total_time}
        value={play_time}
        onChange={(_, value) => on_playtime_slider_change(value as number)}
      />
      <Box display="flex" justifyContent="space-between">
        <Typography variant="overline">{current_time}</Typography>
        <Typography variant="overline">-{remaining_time}</Typography>
      </Box>
      <Box sx={{ width: "fit-content", margin: "auto" }}>
        <IconButton>
          <FastRewindRounded fontSize="large" />
        </IconButton>
        <IconButton onClick={() => { setPlaying(!playing); }}>
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
  );
}

export default Player;