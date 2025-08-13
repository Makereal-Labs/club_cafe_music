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
      main: '#6969b0',
    },
    secondary: {
      main: '#e6ca00',
    },
    error: {
      main: red.A400,
    },
    background: {
      default: grey[200],
      paper: grey[200],
    },
  },
});

const dark_theme = createTheme({
  cssVariables: true,
  palette: {
    mode: 'dark',
    primary: {
      main: '#686bc1',
    },
    secondary: {
      main: '#eccd01',
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