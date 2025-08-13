import { DarkMode, LightMode } from "@mui/icons-material";
import { IconButton } from "@mui/material";
import { ThemeId } from "./theme";

type ThemeToggleProps = {
  value: ThemeId,
  onClick: () => void,
};

function ThemeToggle(props: ThemeToggleProps) {
  return (
    <IconButton onClick={props.onClick}>
      {
        (props.value == ThemeId.Light) ? <LightMode /> : <DarkMode />
      }
    </IconButton>
  );
}

export default ThemeToggle;