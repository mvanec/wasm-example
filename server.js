const fileUpload = require('express-fileupload');
const express = require('express');
const { convert_image } = require('./pkg/wasm_example');

const app = express();
app.use(fileUpload());
const port = 3030;

app.get('/', (req, res) => {
  res.send(`
    <h2>With <code>"express"</code> npm package</h2>
    <form action="/api/upload" enctype="multipart/form-data" method="post">
    <div>
      <p>Text field title: <input type="text" name="title" /></p>
      <p>File: <input type="file" name="file"/></p>
    </div>
    <input type="submit" value="Upload" />
    </form>
  `);
});

app.post('/api/upload', (req, res, next) => {
  const image = convert_image(req.files.file.data)

  res.setHeader('Content-disposition', 'attachment; filename="meme.png"');
  res.setHeader('Content-type', 'image/png');
    res.send(image);
  });

app.listen(port, () => {
  console.log(`Example app listening on port ${port}`)
})
