(function () {
  "use strict";

  const STORAGE_KEY = "ccswarm-order-desk:v1";
  const stages = ["plan", "sangha", "implement", "review"];
  const seeds = [
    {
      id: "order-a",
      title: "Build static habit tracker",
      status: "ready",
      stageIndex: 4,
      recoveryPoints: 1,
    },
    {
      id: "order-b",
      title: "Add partial run recovery",
      status: "partial",
      stageIndex: 2,
      recoveryPoints: 2,
    },
  ];

  const els = {
    ready: document.querySelector("#summary-ready"),
    ordersMetric: document.querySelector("#metric-orders"),
    partialMetric: document.querySelector("#metric-partial"),
    recoveryMetric: document.querySelector("#metric-recovery"),
    orders: document.querySelector("#orders"),
    form: document.querySelector("#order-form"),
    input: document.querySelector("#order-title"),
    error: document.querySelector("#form-error"),
    reset: document.querySelector("#reset-demo"),
    template: document.querySelector("#order-template"),
  };

  let state = loadState();

  function newId() {
    if (window.crypto && typeof window.crypto.randomUUID === "function") {
      return window.crypto.randomUUID();
    }
    const random = Math.floor(Math.random() * 1_000_000).toString(36);
    return `order-${Date.now().toString(36)}-${random}`;
  }

  function loadState() {
    try {
      const raw = window.localStorage.getItem(STORAGE_KEY);
      if (!raw) {
        return { orders: seeds };
      }
      const parsed = JSON.parse(raw);
      if (!parsed || !Array.isArray(parsed.orders)) {
        return { orders: seeds };
      }
      return {
        orders: parsed.orders
          .filter((order) => order && typeof order.title === "string")
          .map((order) => ({
            id: String(order.id || newId()),
            title: order.title.slice(0, 80),
            status: normalizeStatus(order.status),
            stageIndex: clampNumber(order.stageIndex, 0, stages.length),
            recoveryPoints: clampNumber(order.recoveryPoints, 0, 99),
          })),
      };
    } catch (_error) {
      return { orders: seeds };
    }
  }

  function saveState() {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  }

  function normalizeStatus(value) {
    return ["queued", "running", "partial", "ready"].includes(value) ? value : "queued";
  }

  function clampNumber(value, min, max) {
    const numeric = Number(value);
    if (!Number.isFinite(numeric)) {
      return min;
    }
    return Math.min(max, Math.max(min, Math.trunc(numeric)));
  }

  function render() {
    const readyCount = state.orders.filter((order) => order.status === "ready").length;
    const partialCount = state.orders.filter((order) => order.status === "partial").length;
    const recoveryCount = state.orders.reduce((sum, order) => sum + order.recoveryPoints, 0);

    els.ready.textContent = String(readyCount);
    els.ordersMetric.textContent = String(state.orders.length);
    els.partialMetric.textContent = String(partialCount);
    els.recoveryMetric.textContent = String(recoveryCount);

    els.orders.replaceChildren(...state.orders.map(renderOrder));
  }

  function renderOrder(order) {
    const node = els.template.content.firstElementChild.cloneNode(true);
    node.dataset.orderId = order.id;
    node.querySelector("h3").textContent = order.title;

    const pill = node.querySelector(".status-pill");
    pill.textContent = order.status;
    pill.classList.toggle("ready", order.status === "ready");
    pill.classList.toggle("partial", order.status === "partial");

    const stageList = node.querySelector(".stage-list");
    stageList.replaceChildren(
      ...stages.map((stage, index) => {
        const item = document.createElement("li");
        item.textContent = stage;
        item.classList.toggle("done", index < order.stageIndex);
        return item;
      })
    );

    node.querySelector(".recovery-note").textContent =
      order.status === "partial"
        ? `Recovery point saved at ${stages[order.stageIndex] || "complete"}`
        : `${order.recoveryPoints} recovery point${order.recoveryPoints === 1 ? "" : "s"} recorded`;

    node.querySelector('[data-action="advance"]').addEventListener("click", () => {
      updateOrder(order.id, (draft) => {
        draft.stageIndex = Math.min(stages.length, draft.stageIndex + 1);
        draft.status = draft.stageIndex >= stages.length ? "ready" : "running";
      });
    });
    node.querySelector('[data-action="partial"]').addEventListener("click", () => {
      updateOrder(order.id, (draft) => {
        draft.status = "partial";
        draft.recoveryPoints += 1;
      });
    });
    node.querySelector('[data-action="retry"]').addEventListener("click", () => {
      updateOrder(order.id, (draft) => {
        draft.status = "running";
        draft.stageIndex = Math.min(draft.stageIndex + 1, stages.length);
      });
    });

    return node;
  }

  function updateOrder(id, mutate) {
    state = {
      orders: state.orders.map((order) => {
        if (order.id !== id) {
          return order;
        }
        const draft = { ...order };
        mutate(draft);
        return draft;
      }),
    };
    saveState();
    render();
  }

  els.form.addEventListener("submit", (event) => {
    event.preventDefault();
    els.error.textContent = "";
    const title = els.input.value.trim();
    if (!title) {
      els.error.textContent = "Describe the order first.";
      return;
    }
    state = {
      orders: [
        {
          id: newId(),
          title,
          status: "queued",
          stageIndex: 0,
          recoveryPoints: 0,
        },
        ...state.orders,
      ],
    };
    els.input.value = "";
    saveState();
    render();
  });

  els.reset.addEventListener("click", () => {
    state = { orders: seeds };
    saveState();
    render();
  });

  render();
})();
