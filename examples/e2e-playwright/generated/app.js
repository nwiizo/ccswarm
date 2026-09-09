"use strict";

const STORAGE_KEY = "ccswarm-order-desk-v1";
const STAGES = ["plan", "sangha", "implement", "review"];

function createOrder(title, id = globalThis.crypto.randomUUID()) {
  if (typeof title !== "string" || !title.trim()) {
    throw new Error("Describe the order first.");
  }
  return { id, title: title.trim(), status: "queued", stage: null };
}

function advanceOrder(order) {
  if (order.status !== "queued" && order.status !== "running") {
    throw new Error("This order cannot advance. Retry a partial order first.");
  }
  const next = Math.min(STAGES.indexOf(order.stage) + 1, STAGES.length - 1);
  return {
    ...order,
    stage: STAGES[next],
    status: next === STAGES.length - 1 ? "ready" : "running",
  };
}

function markPartial(order) {
  if (order.status !== "running") {
    throw new Error("Only a running order can be marked partial.");
  }
  return { ...order, status: "partial" };
}

function retryOrder(order) {
  if (order.status !== "partial") {
    throw new Error("Only a partial order can be retried.");
  }
  return { ...order, status: "running" };
}

function seedOrders() {
  return [
    {
      id: "demo-002",
      title: "Build the customer onboarding flow",
      status: "partial",
      stage: "implement",
    },
    {
      id: "demo-001",
      title: "Prepare the weekly release notes",
      status: "ready",
      stage: "review",
    },
  ];
}

function readOrders(storage) {
  const saved = storage.getItem(STORAGE_KEY);
  if (saved === null) return seedOrders();

  let orders;
  try {
    orders = JSON.parse(saved);
  } catch (error) {
    throw new Error("Saved orders contain unreadable JSON.", { cause: error });
  }

  const ids = new Set();
  if (
    !Array.isArray(orders) ||
    !orders.every((order) => {
      if (
        !order ||
        typeof order.id !== "string" ||
        !order.id.trim() ||
        ids.has(order.id) ||
        typeof order.title !== "string" ||
        !order.title.trim()
      ) {
        return false;
      }
      ids.add(order.id);
      if (order.status === "queued") return order.stage === null;
      if (order.status === "ready") return order.stage === "review";
      return (
        (order.status === "running" || order.status === "partial") &&
        STAGES.includes(order.stage)
      );
    })
  ) {
    throw new Error("Saved orders have an invalid format.");
  }

  return orders.map(({ id, title, status, stage }) => ({
    id,
    title,
    status,
    stage,
  }));
}

function saveOrders(storage, orders) {
  storage.setItem(STORAGE_KEY, JSON.stringify(orders));
}

function startDesk() {
  const form = document.querySelector("#order-form");
  const input = document.querySelector("#order-title");
  const formError = document.querySelector("#form-error");
  const storageError = document.querySelector("#storage-error");
  const storageStatus = document.querySelector("#storage-status");
  const actionError = document.querySelector("#action-error");
  const announcement = document.querySelector("#announcement");
  const list = document.querySelector("#order-list");
  const template = document.querySelector("#order-template");
  const filters = document.querySelector("#order-filters");
  let orders = [];
  let filter = "all";
  let storage;
  let canSave = false;

  function showStorageError(message) {
    storageError.textContent = message;
    storageError.hidden = false;
    storageStatus.textContent = "Not saved · this tab only";
    storageStatus.dataset.state = "error";
  }

  function persist() {
    if (!canSave) return;
    try {
      saveOrders(storage, orders);
      storageError.hidden = true;
      storageError.textContent = "";
      storageStatus.textContent = "Saved in this browser";
      storageStatus.dataset.state = "saved";
    } catch (error) {
      showStorageError(
        `Orders could not be saved. Changes remain in this tab and may be lost on reload. Check browser storage permissions or available space; the next change will retry saving. ${error.message}`,
      );
    }
  }

  try {
    storage = window.localStorage;
    orders = readOrders(storage);
    canSave = true;
  } catch (error) {
    showStorageError(
      `Saved orders could not be read. You can work in this tab, but changes will not be saved. Existing storage has been left untouched. Check browser storage settings or the saved data, then reload. ${error.message}`,
    );
  }
  persist();

  function render() {
    const ready = orders.filter((order) => order.status === "ready").length;
    const partial = orders.filter((order) => order.status === "partial").length;
    document.querySelector("#metric-orders").textContent = orders.length;
    document.querySelector("#metric-partial").textContent = partial;
    document.querySelector("#summary-ready").textContent = ready;
    document.querySelector("#sidebar-order-count").textContent = orders.length;
    document.querySelector("#queue-count").textContent = orders.length;
    document.querySelector("#queue-summary").textContent = partial
      ? `${partial} ${partial === 1 ? "order needs" : "orders need"} a second look.`
      : "Everything is moving. Keep good work flowing.";

    const visible = orders.filter(
      (order) => filter === "all" || order.status === filter,
    );
    const fragment = document.createDocumentFragment();
    for (const [index, order] of visible.entries()) {
      const card = template.content.firstElementChild.cloneNode(true);
      card.dataset.orderId = order.id;
      card.dataset.status = order.status;
      const heading = card.querySelector("h3");
      heading.textContent = order.title;
      heading.id = `order-heading-${index}`;
      card.setAttribute("aria-labelledby", heading.id);
      card.querySelector(".order-reference").textContent =
        `ORD / ${order.id.slice(-6).toUpperCase()}`;
      const pill = card.querySelector(".status-pill");
      pill.textContent = order.status;
      pill.dataset.status = order.status;

      const stageIndex = STAGES.indexOf(order.stage);
      const steps = card.querySelector(".order-stages");
      steps.setAttribute(
        "aria-label",
        `Workflow. ${order.stage ? `Current stage: ${order.stage}.` : "Not started."}`,
      );
      STAGES.forEach((stage, index) => {
        const step = document.createElement("li");
        const complete = order.status === "ready" || index < stageIndex;
        const current = index === stageIndex;
        step.textContent = stage;
        step.className = complete ? "complete" : current ? "current" : "";
        if (current) step.setAttribute("aria-current", "step");
        steps.append(step);
      });

      const note = card.querySelector(".order-note");
      if (order.status === "partial") {
        note.classList.add("recovery-note");
        note.textContent = `Recovery point saved · ${order.stage}. Retry resumes here.`;
      } else if (order.status === "ready") {
        note.textContent = "Review reached. This demo order is ready.";
      } else if (order.status === "queued") {
        note.textContent = "In the queue. Advance to start planning.";
      } else {
        note.textContent = `At ${order.stage}. Advance when you’re ready for the next step.`;
      }

      card.querySelector('[data-action="advance"]').hidden = ![
        "queued",
        "running",
      ].includes(order.status);
      card.querySelector('[data-action="partial"]').hidden =
        order.status !== "running";
      card.querySelector('[data-action="retry"]').hidden =
        order.status !== "partial";
      fragment.append(card);
    }
    list.replaceChildren(fragment);
    const empty = document.querySelector("#empty-state");
    empty.hidden = visible.length > 0;
    empty.querySelector("h3").textContent = orders.length
      ? `No ${filter} orders.`
      : "Your desk is clear.";
    empty.querySelector("p").textContent = orders.length
      ? "Choose All orders to see the rest of your work."
      : "Add a work order above to get things moving.";
  }

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    try {
      const order = createOrder(input.value);
      orders.unshift(order);
      formError.textContent = "";
      input.removeAttribute("aria-invalid");
      input.value = "";
      setFilter("all");
      persist();
      announcement.textContent = `Added ${order.title}. Order queued.`;
    } catch (error) {
      formError.textContent = error.message;
      input.setAttribute("aria-invalid", "true");
    }
    input.focus();
  });

  input.addEventListener("input", () => {
    if (input.value.trim()) {
      formError.textContent = "";
      input.removeAttribute("aria-invalid");
    }
  });

  function setFilter(value) {
    filter = value;
    for (const button of filters.querySelectorAll("button")) {
      button.setAttribute(
        "aria-pressed",
        String(button.dataset.filter === filter),
      );
    }
    render();
  }

  filters.addEventListener("click", (event) => {
    const button = event.target.closest("button[data-filter]");
    if (button) setFilter(button.dataset.filter);
  });

  list.addEventListener("click", (event) => {
    const button = event.target.closest("button[data-action]");
    if (!button) return;
    const card = button.closest(".order-card");
    const index = orders.findIndex(
      (order) => order.id === card.dataset.orderId,
    );
    if (index === -1) {
      actionError.textContent =
        "This order is no longer available. Reload the desk to continue.";
      return;
    }
    const order = orders[index];
    const action = button.dataset.action;
    const cardIndex = [...list.children].indexOf(card);
    try {
      if (action === "delete") {
        orders.splice(index, 1);
        announcement.textContent = `Deleted ${order.title}.`;
      } else {
        const actions = {
          advance: advanceOrder,
          partial: markPartial,
          retry: retryOrder,
        };
        if (!actions[action]) throw new Error("This action is unavailable.");
        orders[index] = actions[action](order);
        announcement.textContent = `${order.title}: ${orders[index].status} at ${orders[index].stage}.`;
      }
      actionError.textContent = "";
      persist();
      render();
      // Rendering replaces buttons; restore keyboard focus to the same order.
      const remaining = [...list.children];
      const next =
        remaining.find((item) => item.dataset.orderId === order.id) ??
        remaining[Math.min(cardIndex, remaining.length - 1)];
      const sameAction = next?.querySelector(
        `button[data-action="${action}"]:not([hidden])`,
      );
      (
        sameAction ??
        next?.querySelector("button:not([hidden])") ??
        input
      ).focus();
    } catch (error) {
      actionError.textContent = error.message;
    }
  });

  render();
}

if (typeof document !== "undefined") startDesk();

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    STORAGE_KEY,
    createOrder,
    advanceOrder,
    markPartial,
    retryOrder,
    seedOrders,
    readOrders,
    saveOrders,
  };
}
