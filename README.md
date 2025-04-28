<p align="center">
  <a href="https://picaos.com">
    <img alt="Pica Logo" src="./resources/images/banner.png?" style="border-radius: 10px;">
  </a>
</p>

<p align="center"><b>Pica, The AI Integrations Solution</b></p>

<p align="center">
  <b>
    <a href="https://www.picaos.com/">Website</a>
    ·
    <a href="https://docs.picaos.com">Documentation</a>
    ·
    <a href="https://app.picaos.com">Dashboard</a>
    ·
    <a href="https://docs.picaos.com/changelog">Changelog</a>
    ·
    <a href="https://x.com/picahq">X</a>
    ·
    <a href="https://www.linkedin.com/company/picahq">LinkedIn</a>
  </b>
</p>

---

Pica gives every builder instant, reliable access to the tools they need—no keys, no configs, no headaches.

## Why Pica?

Pica simplifies AI agent development with our four core products:

✅ OneTool – Connect agents to [100+ APIs and tools](https://app.picaos.com/tools) with a single SDK. <br/>
✅ AuthKit – Secure authentication for seamless tool integration. <br/>
✅ BuildKit - Empower vibe coding with integrations that work zero-shot.

Pica also provides full logging and action traceability, giving developers complete visibility into their agents’ decisions and activities. Our tools simplify building and running AI agents so developers can focus on results.

## Getting started

### Install

```bash
npm install @picahq/ai
```

### Setup

1. Create a new [Pica account](https://app.picaos.com)
2. Create a Connection via the [Dashboard](https://app.picaos.com/connections)
3. Create an [API key](https://app.picaos.com/settings/api-keys)
4. Set the API key as an environment variable: `PICA_SECRET_KEY=<your-api-key>`

### Example Usage

Below is an example demonstrating how to integrate the [Pica OneTool](https://www.npmjs.com/package/@picahq/ai) with the [Vercel AI SDK](https://www.npmjs.com/package/ai) for a GitHub use case:

```typescript
import { openai } from "@ai-sdk/openai";
import { generateText } from "ai";
import { Pica } from "@picahq/ai";
import * as dotenv from "dotenv";
dotenv.config();

const pica = new Pica(process.env.PICA_SECRET_KEY!, {
  connectors: ["*"]
});

async function runAgentTask(message: string): Promise<string> {
  const system = await pica.generateSystemPrompt();

  const { text } = await generateText({
    model: openai("gpt-4.1"),
    system,
    prompt: message,
    tools: { ...pica.oneTool },
    maxSteps: 10,
  });

  return text;
}

runAgentTask("Star the repo picahq/pica with github")
  .then((text) => {
    console.log(text);
  })
  .catch(console.error);
```

[![Try with Replit Badge](https://replit.com/badge?caption=Try%20with%20Replit)](https://replit.com/@picahq/Pica-or-GitHub-Star-Demo)


For more use cases, visit our [Use Cases Library](https://www.picaos.com/community/use-cases) or our [Awesome Pica Repository](https://github.com/picahq/awesome-pica).

### Next.js Integration

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
