const express = require('express')
const app = express()

const port = 3000

app.get('/random', (req, res) => {
  console.log("api called")
  const random = getRandomInt(2)
  res.send(random.toString())
})

function getRandomInt(max) {
    return Math.floor(Math.random() * Math.floor(max));
  }

  
app.listen(port, () => {
  console.log(`Example app listening at http://localhost:${port}`)
})