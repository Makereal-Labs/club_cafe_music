import { NotesOutlined } from "@mui/icons-material";
import { Dialog, DialogContent, DialogTitle, IconButton, Typography } from "@mui/material";
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
          <Markdown
            children={ChangelogMd}
            disallowedElements={["h1"]}
            components={{
              h2: ({ ...props }) => <Typography variant="h4" gutterBottom={true} {...props} />,
              h3: ({ ...props }) => <Typography variant="h5" gutterBottom={true} {...props} />,
              h4: ({ ...props }) => <Typography variant="h6" gutterBottom={true} {...props} />,
              p: ({ ...props }) => <Typography gutterBottom={true} {...props} />,
              ul: ({ ...props }) => <Typography gutterBottom={true} {...props} />,
            }}
          />
        </DialogContent>
      </Dialog>
    </>
  );
}

export default ChangelogView;
