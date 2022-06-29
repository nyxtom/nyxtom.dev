---
title: Cooperative Scheduling in React
published: true
description: Implementing a simple cooperative scheduler in javascript inspired by how React does scheduling for work in coordination with requestIdleCallback.
tags: [javascript]
---

I've been spending some time going through the source of [React](https://reactjs.org) and one of the things that has bugged me is how frameworks like Aurelia or React attempt to manage work in such a way that it doesn't interfere with the rendering lifecycle of the event loop. It is imperative that before we get into the implementation details of work scheduling that we take a moment to understand how the event loop works.

## JavaScript Runtime

Within the javascript runtime it's important to remember that JavaScript is single-threaded. This means that the processing of everything happens in what is somewhat comparable to a game loop (for the game development inclined). Work is queued up, executed and should it be time to render - a render execution is performed.

## Event Loop

One of the best ways to get an idea of what the event loop is doing is to simply read the [specification](https://html.spec.whatwg.org/multipage/webappapis.html#event-loop-processing-model). You may find that the general processing model can be broken down into the following loop:

1. let taskQueue = // one of many task queues
2. From taskQueue t, find the oldest runnable task and remove it from the taskQueue
3. Execute the task onto the call stack runtime
4. Perform a **microtask** checkpoint
5. Update rendering in a window event loop and we have a rendering opportunity
6. If this is a window event loop, no tasks are in the task queues, microtask is empty, and the rendering opportunity is not time yet - then execute an idle period callback

That's pretty much it. Given that each browser has the affordance to implement the **find oldest runnable task** however it likes, this leads to some ability to perform prioritization of some tasks over others.


