<p align="center">
  <a href="https://picaos.com">
    <img alt="Pica Logo" src="./resources/images/banner.png" style="border-radius: 10px;">
  </a>
</p>

<p align="center"><b>The Complete Agentic Infrastructure</b></p>

<p align="center">
  <b>
    <a href="https://www.picaos.com/">Website</a>
    ·
    <a href="https://docs.picaos.com">Documentation</a>
    ·
    <a href="https://www.picaos.com/community">Community Hub</a>
    ·
    <a href="https://www.picaos.com/community/changelog">Changelog</a>
    ·
    <a href="https://x.com/picahq">X</a>
    ·
    <a href="https://www.linkedin.com/company/picahq">LinkedIn</a>
  </b>
</p>

---

Build, deploy, and scale your AI agents with ease. With full access to [100+ APIs and tools](https://www.picaos.com/community/connectors).

Pica makes it simple to build and manage AI agents with four key products:
1. **OneTool**: Connect agents to over 100 APIs and tools with a single SDK.
2. **AuthKit**: Securely manage authentication for tool integration.
3. **Agent**: Create flexible agents that adapt to your needs (coming soon).
4. **AgentFlow**: Enable agents to collaborate and manage tasks automatically (coming soon).

Pica also provides full logging and action traceability, giving developers complete visibility into their agents’ decisions and activities. Our tools simplify building and running AI agents so developers can focus on results.

## Getting started

```bash
npm install @picahq/ai
```

## Setup

1. Create a new [Pica account](https://app.picaos.com)
2. Create a Connection via the [Dashboard](https://app.picaos.com/connections)
3. Create an [API key](https://app.picaos.com/settings/api-keys)
4. Set the API key as an environment variable: `PICA_SECRET_KEY=<your-api-key>`


## Example use cases

Pica provides various SDKs to connect with different LLMs. Below are samples for using the [Pica AI SDK](https://www.npmjs.com/package/@picahq/ai) designed for the [Vercel AI SDK](https://www.npmjs.com/package/ai):

### Express

1. **Install dependencies**

```bash
npm install express @ai-sdk/openai ai @picahq/ai dotenv
```

2. **Create the server**

```typescript
import express from "express";
import { openai } from "@ai-sdk/openai";
import { generateText } from "ai";
import { Pica } from "@picahq/ai";
import * as dotenv from "dotenv";

dotenv.config();

const app = express();
const port = process.env.PORT || 3000;

app.use(express.json());

app.post("/api/ai", async (req, res) => {
  try {
    const { message } = req.body;

    // Initialize Pica
    const pica = new Pica(process.env.PICA_SECRET_KEY);

    // Generate the system prompt
    const systemPrompt = await pica.generateSystemPrompt();

    // Create the stream
    const { text } = await generateText({
      model: openai("gpt-4o"),
      system: systemPrompt,
      tools: { ...pica.oneTool },
      prompt: message,
      maxSteps: 5,
    });

    res.setHeader("Content-Type", "application/json");

    res.status(200).json({ text });
  } catch (error) {
    console.error("Error processing AI request:", error);

    res.status(500).json({ error: "Internal server error" });
  }
});

app.listen(port, () => {
  console.log(`Server is running on port ${port}`);
});

export default app;
```

3. **Test the server**

```bash
curl --location 'http://localhost:3000/api/ai' \
--header 'Content-Type: application/json' \
--data '{
    "message": "What connections do I have access to?"
}'
```

### Next.js

⭐️ You can see a full Next.js demo [here](https://github.com/picahq/onetool-demo)


> For more examples and detailed documentation, check out our [SDK documentation](https://docs.picaos.com/sdk/vercel-ai).

---

## Running Pica locally

> [!IMPORTANT]
> The Pica dashboard is going open source! Stay tuned for the big release 🚀

### Prerequisites

* [Docker](https://docs.docker.com/engine/)
* [Docker Compose](https://docs.docker.com/compose/)

### Step 1: Install the Pica CLI

```sh
npm install -g @picahq/cli
```

### Step 2: Initialize the Pica CLI

To generate the configuration file, run:

```shell
pica init
```

### Step 3: Start the Pica Server

```sh
pica start
```

> All the inputs are required. Seeding is optional, but recommended when running the command for the first time.

##### Example

```Shell
# To start the docker containers
pica start
Enter the IOS Crypto Secret (32 characters long): xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
Do you want to seed? (Y/N) y
```

**The Pica API will be available at `http://localhost:3005` 🚀**

To stop the docker containers, simply run:

```Shell
pica stop
```


## License

Pica is released under the [**GPL-3.0 license**](LICENSE).
