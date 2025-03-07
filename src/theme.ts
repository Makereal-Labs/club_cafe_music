import { createTheme } from '@mui/material/styles';
import { grey, red } from '@mui/material/colors';

export enum ThemeId {
  Light = 1,
  Dark,
}

export function get_theme(theme: ThemeId) {
  if (theme == ThemeId.Light)
    return light_theme;
  return dark_theme;
}

const light_theme = createTheme({
  cssVariables: true,
  palette: {
    mode: 'light',
    primary: {
      main: '#4347b2',
    },
    secondary: {
      main: '#ECCD01',
    },
    error: {
      main: red.A400,
    },
  },
});

const dark_theme = createTheme({
  cssVariables: true,
  palette: {
    mode: 'dark',
    primary: {
      main: '#4347b2',
    },
    secondary: {
      main: '#ECCD01',
    },
    error: {
      main: red.A400,
    },
    background: {
      default: grey[900],
      paper: grey[900],
    },
  },
});