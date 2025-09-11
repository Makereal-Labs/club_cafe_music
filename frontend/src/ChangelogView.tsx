import { NotesOutlined } from "@mui/icons-material";
import { Dialog, DialogContent, DialogContentText, DialogTitle, IconButton, Typography } from "@mui/material";
import { useState } from "react";
import Markdown from "react-markdown";

import ChangelogMd from "../../CHANGELOG.md?raw";

function ChangelogView() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <IconButton onClick={() => setOpen(true)}>
        <NotesOutlined />
      </IconButton>
      <Dialog
        open={open}
        onClose={() => setOpen(false)}
        scroll="body"
      >
        <Markdown
          children={ChangelogMd}
          allowedElements={["h1"]}
          components={{
            h1: ({ ...props }) => <DialogTitle {...props} />
          }}
        />
        <DialogContent>
          <DialogContentText>
            <Markdown
              children={ChangelogMd}
              disallowedElements={["h1"]}
              components={{
                h2: ({ ...props }) => <Typography variant="h4" sx={{ paddingTop: 2 }} {...props} />,
                h3: ({ ...props }) => <Typography variant="h5" sx={{ paddingTop: 2 }} {...props} />,
                h4: ({ ...props }) => <Typography variant="h6" sx={{ paddingTop: 2 }} {...props} />,
              }}
            />
          </DialogContentText>
        </DialogContent>
      </Dialog>
    </>
  );
}

export default ChangelogView;
