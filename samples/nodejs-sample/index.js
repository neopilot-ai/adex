const express = require('express');
const app = express();
const port = 3000;

app.get('/', (req, res) => {
  res.send('Hello from the Node.js sample app!');
});

app.listen(port, () => {
  console.log(`Node.js sample app listening at http://localhost:${port}`);
});
