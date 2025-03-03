import { createTheme } from '@mui/material/styles';
import { grey, red } from '@mui/material/colors';

const theme = createTheme({
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
    }
  },
});

export default theme;