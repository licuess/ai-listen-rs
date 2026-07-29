import { invoke } from "@tauri-apps/api/core";

let sessions = [];
let searchHits = [];
let activeSlug = null;
let saveTimer = null;
let taskPollTimer = null;

const els = {
  newSession: document.querySelector("#newSession"),
  searchInput: document.querySelector("#searchInput"),
  audioDevice: document.querySelector("#audioDevice"),
  audioTestButton: document.querySelector("#audioTestButton"),
  sessionList: document.querySelector("#sessionList"),
  sessionTitle: document.querySelector("#sessionTitle"),
  sessionPath: document.querySelector("#sessionPath"),
  noteEditor: document.querySelector("#noteEditor"),
  screenshotButton: document.querySelector("#screenshotButton"),
  analyzeButton: document.querySelector("#analyzeButton"),
  recordButton: document.querySelector("#recordButton"),
  audioButton: document.querySelector("#audioButton"),
  transcribeButton: document.querySelector("#transcribeButton"),
  summaryButton: document.querySelector("#summaryButton"),
  indexButton: document.querySelector("#indexButton"),
  exportButton: document.querySelector("#exportButton"),
  exportMenu: document.querySelector(".export-menu"),
  saveButton: document.querySelector("#saveButton"),
  screenshotCount: document.querySelector("#screenshotCount"),
  recordingCount: document.querySelector("#recordingCount"),
  audioCount: document.querySelector("#audioCount"),
  transcriptCount: document.querySelector("#transcriptCount"),
  summaryBox: document.querySelector("#summaryBox"),
  statusText: document.querySelector("#statusText"),
  taskList: document.querySelector("#taskList"),
  transcriptPanel: document.querySelector("#transcriptPanel"),
  transcriptLive: document.querySelector("#transcriptLive"),
  screenshotPreviewSection: document.querySelector("#screenshotPreviewSection"),
  screenshotPreviewCount: document.querySelector("#screenshotPreviewCount"),
  screenshotGrid: document.querySelector("#screenshotGrid"),
  screenshotViewer: document.querySelector("#screenshotViewer"),
  viewerOverlay: document.querySelector("#viewerOverlay"),
  viewerImg: document.querySelector("#viewerImg"),
  viewerClose: document.querySelector("#viewerClose"),
  viewerPrev: document.querySelector("#viewerPrev"),
  viewerNext: document.querySelector("#viewerNext"),
  viewerInfo: document.querySelector("#viewerInfo"),
  // 登录
  loginOverlay: document.querySelector("#loginOverlay"),
  loginMode: document.querySelector("#loginMode"),
  registerMode: document.querySelector("#registerMode"),
  loginTabs: document.querySelectorAll("#loginMode .login-tab"),
  registerTabs: document.querySelectorAll("#registerMode .login-tab"),
  loginFormEmail: document.querySelector("#loginFormEmail"),
  loginFormPhone: document.querySelector("#loginFormPhone"),
  loginFormUsername: document.querySelector("#loginFormUsername"),
  registerFormEmail: document.querySelector("#registerFormEmail"),
  registerFormPhone: document.querySelector("#registerFormPhone"),
  loginEmailProvider: document.querySelector("#loginEmailProvider"),
  loginEmail: document.querySelector("#loginEmail"),
  loginEmailPassword: document.querySelector("#loginEmailPassword"),
  loginEmailBtn: document.querySelector("#loginEmailBtn"),
  loginPhone: document.querySelector("#loginPhone"),
  loginPhoneCode: document.querySelector("#loginPhoneCode"),
  loginPhoneSendCode: document.querySelector("#loginPhoneSendCode"),
  loginPhoneBtn: document.querySelector("#loginPhoneBtn"),
  loginUsername: document.querySelector("#loginUsername"),
  loginUsernamePassword: document.querySelector("#loginUsernamePassword"),
  loginUsernameBtn: document.querySelector("#loginUsernameBtn"),
  goRegister: document.querySelector("#goRegister"),
  goRegisterVip: document.querySelector("#goRegisterVip"),
  goLogin: document.querySelector("#goLogin"),
  regEmail: document.querySelector("#regEmail"),
  regEmailProvider: document.querySelector("#regEmailProvider"),
  regEmailCode: document.querySelector("#regEmailCode"),
  regEmailSendCode: document.querySelector("#regEmailSendCode"),
  regPassword: document.querySelector("#regPassword"),
  regPasswordConfirm: document.querySelector("#regPasswordConfirm"),
  registerEmailBtn: document.querySelector("#registerEmailBtn"),
  regPhone: document.querySelector("#regPhone"),
  regPhoneCode: document.querySelector("#regPhoneCode"),
  regPhoneSendCode: document.querySelector("#regPhoneSendCode"),
  regPhonePassword: document.querySelector("#regPhonePassword"),
  registerPhoneBtn: document.querySelector("#registerPhoneBtn"),
  // 设置
  settingsBtn: document.querySelector("#settingsBtn"),
  settingsOverlay: document.querySelector("#settingsOverlay"),
  settingsClose: document.querySelector("#settingsClose"),
  settingsApiKey: document.querySelector("#settingsApiKey"),
  settingsContextWindow: document.querySelector("#settingsContextWindow"),
  settingsTokenLimit: document.querySelector("#settingsTokenLimit"),
  settingsSaveBtn: document.querySelector("#settingsSaveBtn"),
  // Token 报表
  tokenReportBtn: document.querySelector("#tokenReportBtn"),
  tokenReportOverlay: document.querySelector("#tokenReportOverlay"),
  tokenReportClose: document.querySelector("#tokenReportClose"),
  tokenTotal: document.querySelector("#tokenTotal"),
  tokenPrompt: document.querySelector("#tokenPrompt"),
  tokenCompletion: document.querySelector("#tokenCompletion"),
  tokenLimitDisplay: document.querySelector("#tokenLimitDisplay"),
  tokenPercentDisplay: document.querySelector("#tokenPercentDisplay"),
  tokenLimitFill: document.querySelector("#tokenLimitFill"),
  tokenByOperation: document.querySelector("#tokenByOperation"),
  tokenRecent: document.querySelector("#tokenRecent"),
  mainShell: document.querySelector("#mainShell")
};

els.newSession.addEventListener("click", createSession);
els.searchInput.addEventListener("input", () => loadSessions(els.searchInput.value));
els.saveButton.addEventListener("click", saveCurrentNote);
els.noteEditor.addEventListener("input", () => {
  window.clearTimeout(saveTimer);
  saveTimer = window.setTimeout(saveCurrentNote, 600);
});
els.screenshotButton.addEventListener("click", captureScreenshot);
els.analyzeButton.addEventListener("click", analyzeScreenshot);
els.recordButton.addEventListener("click", toggleRecording);
els.audioButton.addEventListener("click", toggleAudio);
els.audioTestButton.addEventListener("click", testAudioDevice);
els.transcribeButton.addEventListener("click", transcribeLatestAudio);
els.summaryButton.addEventListener("click", summarize);
els.indexButton.addEventListener("click", rebuildIndex);
els.exportButton.addEventListener("click", () => {
  els.exportMenu.hidden = !els.exportMenu.hidden;
});
els.exportMenu.addEventListener("click", (e) => {
  const format = e.target.dataset.format;
  if (format) {
    els.exportMenu.hidden = true;
    exportNote(format);
  }
});
// 点击其他区域关闭导出菜单
document.addEventListener("click", (e) => {
  if (!e.target.closest(".export-group")) {
    els.exportMenu.hidden = true;
  }
});

// 截图查看器事件
els.viewerClose.addEventListener("click", closeViewer);
els.viewerOverlay.addEventListener("click", closeViewer);
els.viewerPrev.addEventListener("click", () => navigateViewer(-1));
els.viewerNext.addEventListener("click", () => navigateViewer(1));
document.addEventListener("keydown", (e) => {
  if (els.screenshotViewer.hidden) return;
  if (e.key === "Escape") closeViewer();
  if (e.key === "ArrowLeft") navigateViewer(-1);
  if (e.key === "ArrowRight") navigateViewer(1);
});

// ========== 登录逻辑 ==========
function checkLoginState() {
  const saved = localStorage.getItem("ai_listen_login");
  if (saved) {
    els.loginOverlay.hidden = true;
    els.mainShell.hidden = false;
    return true;
  }
  return false;
}

// 登录 Tab 切换
els.loginTabs.forEach((tab) => {
  tab.addEventListener("click", () => {
    els.loginTabs.forEach((t) => t.classList.remove("active"));
    tab.classList.add("active");
    const target = tab.dataset.tab;
    els.loginFormEmail.hidden = target !== "email";
    els.loginFormUsername.hidden = target !== "username";
  });
});

// 注册 Tab 切换
els.registerTabs.forEach((tab) => {
  tab.addEventListener("click", () => {
    els.registerTabs.forEach((t) => t.classList.remove("active"));
    tab.classList.add("active");
    const target = tab.dataset.rtab;
    els.registerFormEmail.hidden = target !== "email";
    els.registerFormPhone.hidden = target !== "phone";
  });
});

// 登录/注册模式切换
els.goRegister.addEventListener("click", () => {
  els.loginMode.hidden = true;
  els.registerMode.hidden = false;
});
els.goRegisterVip.addEventListener("click", () => {
  els.loginMode.hidden = true;
  els.registerMode.hidden = false;
});
els.goLogin.addEventListener("click", () => {
  els.registerMode.hidden = true;
  els.loginMode.hidden = false;
});

// 密码显示/隐藏
document.querySelectorAll(".password-toggle").forEach((btn) => {
  btn.addEventListener("click", () => {
    const input = btn.parentElement.querySelector("input");
    if (input.type === "password") {
      input.type = "text";
      btn.style.opacity = "1";
    } else {
      input.type = "password";
      btn.style.opacity = "0.5";
    }
  });
});

function doLogin(method, info) {
  const remember = document.getElementById("loginRemember")?.checked;
  localStorage.setItem("ai_listen_login", JSON.stringify({ method, info, time: Date.now(), remember }));
  els.loginOverlay.hidden = true;
  els.mainShell.hidden = false;
  setStatus(`已登录（${method}）`);
}

els.loginEmailBtn.addEventListener("click", async () => {
  const provider = els.loginEmailProvider.value;
  const account = els.loginEmail.value.trim();
  const password = els.loginEmailPassword.value;
  if (!account || !password) { setStatus("请填写账号和密码"); return; }
  const fullEmail = account.includes("@") ? account : account + provider;
  setStatus("正在登录...");
  try {
    const result = await invoke("login_email", { email: fullEmail, password });
    if (result.success) {
      doLogin("email", { email: fullEmail, user: result.user });
    } else {
      setStatus(result.message);
    }
  } catch (error) {
    setStatus("登录失败：" + String(error));
  }
});

els.loginPhoneBtn.addEventListener("click", async () => {
  const phone = els.loginPhone.value.trim();
  const code = els.loginPhoneCode.value.trim();
  if (!phone || !code) { setStatus("请填写手机号和验证码"); return; }
  setStatus("正在验证...");
  try {
    await invoke("verify_code", { target: phone, code });
    const result = await invoke("login_phone", { phone, password: code });
    if (result.success) {
      doLogin("phone", { phone, user: result.user });
    } else {
      setStatus(result.message);
    }
  } catch (error) {
    setStatus(String(error));
  }
});

els.loginPhoneSendCode.addEventListener("click", async () => {
  const phone = els.loginPhone.value.trim();
  if (!phone) { setStatus("请先输入手机号"); return; }
  setStatus("正在发送验证码...");
  try {
    const result = await invoke("send_phone_code", { phone });
    setStatus(result);
    els.loginPhoneSendCode.disabled = true;
    let countdown = 60;
    els.loginPhoneSendCode.textContent = `${countdown}s`;
    const timer = setInterval(() => {
      countdown--;
      els.loginPhoneSendCode.textContent = countdown > 0 ? `${countdown}s` : "发送验证码";
      if (countdown <= 0) {
        clearInterval(timer);
        els.loginPhoneSendCode.disabled = false;
      }
    }, 1000);
  } catch (error) {
    setStatus(String(error));
  }
});

els.loginUsernameBtn.addEventListener("click", async () => {
  const username = els.loginUsername.value.trim();
  const password = els.loginUsernamePassword.value;
  if (!username || !password) { setStatus("请填写用户名和密码"); return; }
  setStatus("正在登录...");
  try {
    const result = await invoke("login_username", { identifier: username, password });
    if (result.success) {
      doLogin("username", { username, user: result.user });
    } else {
      setStatus(result.message);
    }
  } catch (error) {
    setStatus("登录失败：" + String(error));
  }
});

document.querySelectorAll(".social-btn").forEach((btn) => {
  btn.addEventListener("click", async () => {
    const method = btn.dataset.method;
    setStatus(`正在跳转到${btn.textContent}登录...`);
    try {
      // 调用后端获取 OAuth 授权 URL
      const authUrl = await invoke("get_social_auth_url", { provider: method });
      if (authUrl) {
        // 在浏览器中打开授权页面
        await invoke("open_url", { url: authUrl });
        setStatus(`请在浏览器中完成${btn.textContent}授权`);
      } else {
        // 降级：直接登录（开发模式）
        doLogin(method, { provider: method });
      }
    } catch (error) {
      // 后端未实现时降级处理
      doLogin(method, { provider: method });
    }
  });
});

// ========== 注册逻辑 ==========
els.regEmailSendCode.addEventListener("click", async () => {
  const email = els.regEmail.value.trim() + els.regEmailProvider.value;
  if (!els.regEmail.value.trim()) { setStatus("请输入邮箱账号"); return; }
  setStatus("正在发送验证码...");
  try {
    const result = await invoke("send_email_code", { email });
    setStatus(result);
    els.regEmailSendCode.disabled = true;
    let countdown = 60;
    els.regEmailSendCode.textContent = `${countdown}s`;
    const timer = setInterval(() => {
      countdown--;
      els.regEmailSendCode.textContent = countdown > 0 ? `${countdown}s` : "发送验证码";
      if (countdown <= 0) {
        clearInterval(timer);
        els.regEmailSendCode.disabled = false;
      }
    }, 1000);
  } catch (error) {
    setStatus(String(error));
  }
});

els.registerEmailBtn.addEventListener("click", async () => {
  const email = els.regEmail.value.trim() + els.regEmailProvider.value;
  const code = els.regEmailCode.value.trim();
  const password = els.regPassword.value;
  const confirm = els.regPasswordConfirm.value;
  if (!els.regEmail.value.trim()) { setStatus("请输入邮箱账号"); return; }
  if (!code) { setStatus("请输入验证码"); return; }
  if (password.length < 6 || password.length > 20) { setStatus("密码长度为6-20位"); return; }
  if (password !== confirm) { setStatus("两次密码不一致"); return; }
  setStatus("正在验证...");
  try {
    await invoke("verify_code", { target: email, code });
    setStatus("正在注册...");
    const result = await invoke("register_email", { email, password });
    if (result.success) {
      localStorage.setItem("ai_listen_login", JSON.stringify({ method: "register_email", info: { email }, user: result.user, time: Date.now() }));
      els.loginOverlay.hidden = true;
      els.mainShell.hidden = false;
      setStatus("注册成功，已自动登录");
      await loadAudioDevices();
      await loadSessions();
      await loadTasks();
    } else {
      setStatus(result.message);
    }
  } catch (error) {
    setStatus(String(error));
  }
});

els.regPhoneSendCode.addEventListener("click", async () => {
  const phone = els.regPhone.value.trim();
  if (!phone) { setStatus("请输入手机号"); return; }
  setStatus("正在发送验证码...");
  try {
    const result = await invoke("send_phone_code", { phone });
    setStatus(result);
    els.regPhoneSendCode.disabled = true;
    let countdown = 60;
    els.regPhoneSendCode.textContent = `${countdown}s`;
    const timer = setInterval(() => {
      countdown--;
      els.regPhoneSendCode.textContent = countdown > 0 ? `${countdown}s` : "发送验证码";
      if (countdown <= 0) {
        clearInterval(timer);
        els.regPhoneSendCode.disabled = false;
      }
    }, 1000);
  } catch (error) {
    setStatus(String(error));
  }
});

els.registerPhoneBtn.addEventListener("click", async () => {
  const phone = els.regPhone.value.trim();
  const code = els.regPhoneCode.value.trim();
  const password = els.regPhonePassword.value;
  if (!phone) { setStatus("请输入手机号"); return; }
  if (!code) { setStatus("请输入验证码"); return; }
  if (password.length < 6 || password.length > 20) { setStatus("密码长度为6-20位"); return; }
  setStatus("正在验证...");
  try {
    await invoke("verify_code", { target: phone, code });
    setStatus("正在注册...");
    const result = await invoke("register_phone", { phone, password });
    if (result.success) {
      localStorage.setItem("ai_listen_login", JSON.stringify({ method: "register_phone", info: { phone }, user: result.user, time: Date.now() }));
      els.loginOverlay.hidden = true;
      els.mainShell.hidden = false;
      setStatus("注册成功，已自动登录");
      await loadAudioDevices();
      await loadSessions();
      await loadTasks();
    } else {
      setStatus(result.message);
    }
  } catch (error) {
    setStatus(String(error));
  }
});

// ========== 设置逻辑 ==========
els.settingsBtn.addEventListener("click", async () => {
  els.settingsOverlay.hidden = false;
  try {
    const s = await invoke("load_settings");
    els.settingsApiKey.value = s.openai_api_key || "";
    els.settingsContextWindow.value = String(s.context_window);
    els.settingsTokenLimit.value = String(s.token_limit);
  } catch { /* ignore */ }
});
els.settingsClose.addEventListener("click", () => { els.settingsOverlay.hidden = true; });
els.settingsSaveBtn.addEventListener("click", async () => {
  try {
    await invoke("save_settings_cmd", {
      key: els.settingsApiKey.value,
      contextWindow: parseInt(els.settingsContextWindow.value) || 4096,
      tokenLimit: parseInt(els.settingsTokenLimit.value) || 100000
    });
    setStatus("设置已保存");
    els.settingsOverlay.hidden = true;
  } catch (error) {
    setStatus("保存失败：" + String(error));
  }
});

// ========== Token 报表逻辑 ==========
els.tokenReportBtn.addEventListener("click", async () => {
  els.tokenReportOverlay.hidden = false;
  await refreshTokenReport();
});
els.tokenReportClose.addEventListener("click", () => { els.tokenReportOverlay.hidden = true; });

async function refreshTokenReport() {
  try {
    const report = await invoke("get_token_report");
    const settings = await invoke("load_settings");
    els.tokenTotal.textContent = report.total_tokens.toLocaleString();
    els.tokenPrompt.textContent = report.total_prompt_tokens.toLocaleString();
    els.tokenCompletion.textContent = report.total_completion_tokens.toLocaleString();
    els.tokenLimitDisplay.textContent = settings.token_limit.toLocaleString();
    const percent = settings.token_limit > 0 ? Math.min(100, (report.total_tokens / settings.token_limit * 100)).toFixed(1) : 0;
    els.tokenPercentDisplay.textContent = percent + "%";
    els.tokenLimitFill.style.width = percent + "%";
    els.tokenLimitFill.classList.toggle("over-limit", parseFloat(percent) > 90);

    // 按操作类型
    els.tokenByOperation.innerHTML = "";
    if (report.by_operation.length === 0) {
      els.tokenByOperation.innerHTML = '<div style="color:#a0a9a4;font-size:13px;padding:8px;">暂无数据</div>';
    } else {
      report.by_operation.forEach((op) => {
        const div = document.createElement("div");
        div.className = "token-op-item";
        div.innerHTML = `<strong>${op.operation}</strong><span>${op.count} 次 · ${op.total_tokens.toLocaleString()} tokens</span>`;
        els.tokenByOperation.appendChild(div);
      });
    }

    // 最近记录
    els.tokenRecent.innerHTML = "";
    if (report.recent.length === 0) {
      els.tokenRecent.innerHTML = '<div style="color:#a0a9a4;font-size:13px;padding:8px;">暂无记录</div>';
    } else {
      report.recent.forEach((r) => {
        const div = document.createElement("div");
        div.className = "token-recent-item";
        div.innerHTML = `<span class="op-name">${r.operation}</span><span>${r.total_tokens.toLocaleString()} tokens</span><span>${formatTime(parseInt(r.timestamp))}</span>`;
        els.tokenRecent.appendChild(div);
      });
    }
  } catch (error) {
    setStatus("加载 Token 报表失败：" + String(error));
  }
}

// 检查登录状态，未登录则显示登录界面
if (!checkLoginState()) {
  els.mainShell.hidden = true;
} else {
  await loadAudioDevices();
  try {
    await loadSessions();
  } catch (error) {
    setStatus("加载会议列表失败：" + String(error));
  }
  try {
    await loadTasks();
  } catch {
    renderTasks([]);
  }
}

async function loadSessions(query = "") {
  setStatus("读取会议列表");
  searchHits = [];
  if (query.trim()) {
    searchHits = await invoke("search_index", { query });
    sessions = searchHits.map((hit) => hit.session);
  } else {
    sessions = await invoke("list_sessions");
  }
  renderSessions();

  if (!activeSlug && sessions.length) {
    await selectSession(sessions[0].slug);
  } else if (!sessions.length && !query.trim()) {
    // 没有会议时自动创建一个默认会议
    try {
      await invoke("create_session", { title: "我的会议" });
      await loadSessions();
    } catch {
      renderEmpty();
    }
  } else if (!sessions.length) {
    renderEmpty();
  }
  setStatus("就绪");
}

async function testAudioDevice() {
  setStatus("正在测试麦克风");
  try {
    const result = await invoke("test_input_device", {
      deviceId: els.audioDevice.value || null
    });
    const maxDb = result.max_db == null ? "n/a" : `${result.max_db.toFixed(1)} dB`;
    setStatus(`${result.message}，峰值 ${maxDb}`);
  } catch (error) {
    setStatus(String(error));
  }
}

async function createSession() {
  const title = window.prompt("会议名称", "新会议");
  if (!title?.trim()) return;

  try {
    const session = await invoke("create_session", { title: title.trim() });
    activeSlug = session.slug;
    await loadSessions();
    await selectSession(session.slug);
  } catch (error) {
    setStatus("创建会议失败：" + String(error));
  }
}

async function selectSession(slug) {
  activeSlug = slug;
  try {
    const session = await invoke("read_session", { slug });
    renderEditor(session);
    renderSessions();
  } catch (error) {
    setStatus("加载会议失败：" + String(error));
  }
}

async function saveCurrentNote() {
  if (!activeSlug) return;
  try {
    const session = await invoke("save_note", {
      slug: activeSlug,
      content: els.noteEditor.value
    });
    renderEditor(session);
    setStatus("已保存");
  } catch (error) {
    setStatus("保存失败：" + String(error));
  }
}

async function captureScreenshot() {
  if (!activeSlug) { setStatus("请先新建或选择一个会议"); return; }
  setStatus("正在截图");
  try {
    const session = await invoke("capture_session_screenshot", { slug: activeSlug });
    renderEditor(session);
    setStatus("截图已保存");
  } catch (error) {
    setStatus("截图失败：" + String(error));
  }
}

async function analyzeScreenshot() {
  if (!activeSlug) { setStatus("请先新建或选择一个会议"); return; }
  setStatus("正在识别截图内容");
  try {
    const task = await invoke("analyze_session_screenshot", { slug: activeSlug });
    renderTasks([task]);
    startTaskPolling(task.id);
  } catch (error) {
    setStatus("识别失败：" + String(error));
  }
}

async function exportNote(format) {
  if (!activeSlug) { setStatus("请先新建或选择一个会议"); return; }
  setStatus(`正在导出为 ${format.toUpperCase()}`);
  try {
    const command = `export_session_${format}`;
    const filePath = await invoke(command, { slug: activeSlug });
    setStatus(`已导出到：${filePath}`);
  } catch (error) {
    setStatus(`导出失败：${String(error)}`);
  }
}

async function toggleRecording() {
  if (!activeSlug) { setStatus("请先新建或选择一个会议"); return; }
  const current = sessions.find((session) => session.slug === activeSlug);
  setStatus(current?.is_recording ? "正在停止录屏" : "正在开始录屏");

  try {
    const session = current?.is_recording
      ? await invoke("stop_session_recording", { slug: activeSlug })
      : await invoke("start_session_recording", { slug: activeSlug });

    renderEditor(session);
    await loadSessions(els.searchInput.value);
    setStatus(session.is_recording ? "录屏中" : "录屏已停止");
  } catch (error) {
    setStatus("录屏操作失败：" + String(error));
  }
}

async function toggleAudio() {
  if (!activeSlug) { setStatus("请先新建或选择一个会议"); return; }
  const current = sessions.find((session) => session.slug === activeSlug);
  setStatus(current?.is_audio_recording ? "正在停止录音" : "正在开始录音");

  try {
    const session = current?.is_audio_recording
      ? await invoke("stop_session_audio", { slug: activeSlug })
      : await invoke("start_session_audio", {
          slug: activeSlug,
          deviceId: els.audioDevice.value || null
        });

    renderEditor(session);
    await loadSessions(els.searchInput.value);
    setStatus(session.is_audio_recording ? "录音中" : "录音已停止");
  } catch (error) {
    setStatus("录音操作失败：" + String(error));
  }
}

async function transcribeLatestAudio() {
  if (!activeSlug) { setStatus("请先新建或选择一个会议"); return; }
  setStatus("已加入转写队列");
  try {
    const task = await invoke("enqueue_transcribe_latest_audio", { slug: activeSlug });
    renderTasks([task]);
    startTaskPolling(task.id);
  } catch (error) {
    setStatus(String(error));
  }
}

async function loadAudioDevices() {
  try {
    const devices = await invoke("list_input_devices");
    els.audioDevice.replaceChildren(
      ...devices.map((device) => {
        const option = document.createElement("option");
        option.value = device.id;
        option.textContent = device.name;
        return option;
      })
    );
  } catch (error) {
    const option = document.createElement("option");
    option.value = "";
    option.textContent = "系统默认输入";
    els.audioDevice.replaceChildren(option);
    setStatus(String(error));
  }
}

async function summarize() {
  if (!activeSlug) { setStatus("请先新建或选择一个会议"); return; }
  setStatus("正在生成摘要");
  try {
    els.summaryBox.textContent = await invoke("summarize_session", { slug: activeSlug });
    setStatus("摘要已更新");
  } catch (error) {
    setStatus("摘要失败：" + String(error));
  }
}

async function rebuildIndex() {
  setStatus("正在刷新索引");
  try {
    const stats = await invoke("rebuild_index");
    setStatus(`索引已刷新：${stats.sessions} 个会议，${stats.terms} 个词项`);
  } catch (error) {
    setStatus("索引刷新失败：" + String(error));
  }
}

async function loadTasks() {
  try {
    renderTasks(await invoke("list_tasks"));
  } catch {
    renderTasks([]);
  }
}

function startTaskPolling(taskId) {
  window.clearInterval(taskPollTimer);
  let lastPartialLen = 0;
  taskPollTimer = window.setInterval(async () => {
    try {
      const task = await invoke("task_status", { task_id: taskId });
      renderTasks([task]);
      setStatus(task.message);

      if (task.partial && task.partial.length > lastPartialLen) {
        els.transcriptPanel.hidden = false;
        els.transcriptLive.textContent = task.partial;
        els.transcriptLive.scrollTop = els.transcriptLive.scrollHeight;
        lastPartialLen = task.partial.length;
      }

      if (task.state === "done" || task.state === "failed") {
        window.clearInterval(taskPollTimer);
        if (task.state === "done") {
          els.transcriptPanel.hidden = true;
          els.transcriptLive.textContent = "";
        }
        await loadSessions(els.searchInput.value);
        if (activeSlug) await selectSession(activeSlug);
      }
    } catch (error) {
      window.clearInterval(taskPollTimer);
      setStatus("任务轮询失败：" + String(error));
    }
  }, 1200);
}

function renderSessions() {
  els.sessionList.replaceChildren(
    ...sessions.map((session) => {
      const button = document.createElement("button");
      button.className = `session-item${session.slug === activeSlug ? " active" : ""}`;
      button.innerHTML = `<strong></strong><span></span><span class="snippet"></span>`;
      button.querySelector("strong").textContent = session.title;
      const hit = searchHits.find((item) => item.session.slug === session.slug);
      const spans = button.querySelectorAll("span");
      spans[0].textContent = hit?.updated_at
        ? `${session.slug} · ${formatTime(hit.updated_at)}`
        : session.slug;
      if (hit?.highlighted_snippet) {
        spans[1].innerHTML = hit.highlighted_snippet;
      }
      button.addEventListener("click", async () => {
        await selectSession(session.slug);
        if (hit?.match_text) jumpToMatch(hit.match_text);
      });
      return button;
    })
  );
}

function renderEditor(session) {
  activeSlug = session.slug;
  const index = sessions.findIndex((item) => item.slug === session.slug);
  if (index >= 0) sessions[index] = session;

  els.sessionTitle.textContent = session.title;
  els.sessionPath.textContent = session.path;
  els.noteEditor.value = session.notes;
  els.screenshotCount.textContent = String(session.screenshots.length);
  els.recordingCount.textContent = String(session.recordings.length);
  els.audioCount.textContent = String(session.audio.length);
  els.transcriptCount.textContent = String(session.transcripts.length);
  els.recordButton.textContent = session.is_recording ? "停止" : "录屏";
  els.recordButton.title = session.is_recording ? "停止录屏" : "开始录屏";
  els.recordButton.classList.toggle("recording-active", !!session.is_recording);
  els.audioButton.textContent = session.is_audio_recording ? "停止" : "录音";
  els.audioButton.title = session.is_audio_recording ? "停止录音" : "开始录音";
  els.audioButton.classList.toggle("recording-active", !!session.is_audio_recording);
  renderScreenshotPreview(session);
}

// 截图查看器
let viewerScreenshots = [];
let viewerCurrentIndex = 0;

function renderScreenshotPreview(session) {
  const screenshots = session.screenshots;
  // 存储到模块级变量，确保点击时数据可用
  viewerScreenshots = screenshots;

  if (screenshots.length === 0) {
    els.screenshotPreviewSection.hidden = true;
    return;
  }

  els.screenshotPreviewSection.hidden = false;
  els.screenshotPreviewCount.textContent = String(screenshots.length);

  // 渲染截图网格（最新在前）
  els.screenshotGrid.innerHTML = "";
  const reversed = [...screenshots].reverse();
  reversed.forEach((path, index) => {
    const thumb = document.createElement("div");
    thumb.className = "screenshot-thumb";
    const img = document.createElement("img");
    // 使用 Rust 命令加载图片为 base64 data URL，避免 asset protocol 路径问题
    loadImageAsDataUrl(path).then((dataUrl) => {
      img.src = dataUrl;
    }).catch(() => {
      img.alt = `截图加载失败`;
    });
    img.loading = "lazy";
    img.alt = `截图 ${screenshots.length - index}`;
    thumb.appendChild(img);
    thumb.addEventListener("click", () => openViewer(path));
    els.screenshotGrid.appendChild(thumb);
  });
}

function openViewer(path) {
  viewerCurrentIndex = viewerScreenshots.indexOf(path);
  if (viewerCurrentIndex < 0) viewerCurrentIndex = 0;
  updateViewerImage();
  els.screenshotViewer.hidden = false;
}

function closeViewer() {
  els.screenshotViewer.hidden = true;
  els.viewerImg.src = "";
}

function navigateViewer(direction) {
  viewerCurrentIndex = (viewerCurrentIndex + direction + viewerScreenshots.length) % viewerScreenshots.length;
  updateViewerImage();
}

function updateViewerImage() {
  const path = viewerScreenshots[viewerCurrentIndex];
  loadImageAsDataUrl(path).then((dataUrl) => {
    els.viewerImg.src = dataUrl;
  });
  els.viewerInfo.textContent = `${viewerCurrentIndex + 1} / ${viewerScreenshots.length}`;
}

async function loadImageAsDataUrl(path) {
  return await invoke("load_image_as_data_url", { path });
}

function jumpToMatch(text) {
  const haystack = els.noteEditor.value.toLowerCase();
  const needle = text.toLowerCase();
  const index = haystack.indexOf(needle);
  if (index < 0) return;

  els.noteEditor.focus();
  els.noteEditor.setSelectionRange(index, index + text.length);
  const ratio = index / Math.max(els.noteEditor.value.length, 1);
  els.noteEditor.scrollTop = ratio * els.noteEditor.scrollHeight;
}

function renderTasks(tasks) {
  if (!tasks.length) {
    els.taskList.textContent = "暂无后台任务";
    return;
  }

  els.taskList.replaceChildren(
    ...tasks.slice(-4).map((task) => {
      const item = document.createElement("div");
      item.className = "task-item";
      item.innerHTML = `<strong></strong><span></span><div class="task-progress"><i></i></div><span></span><span></span>`;
      item.querySelector("strong").textContent = task.kind;
      item.querySelectorAll("span")[0].textContent = task.state;
      item.querySelector(".task-progress i").style.width = `${task.progress ?? 0}%`;
      item.querySelectorAll("span")[1].textContent = task.message;
      item.querySelectorAll("span")[2].textContent = task.partial ?? "";
      return item;
    })
  );
}

function formatTime(seconds) {
  return new Date(seconds * 1000).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit"
  });
}

function renderEmpty() {
  activeSlug = null;
  els.sessionTitle.textContent = "选择或新建一个会议";
  els.sessionPath.textContent = "未选择会议";
  els.noteEditor.value = "";
  els.screenshotCount.textContent = "0";
  els.recordingCount.textContent = "0";
  els.audioCount.textContent = "0";
  els.transcriptCount.textContent = "0";
  els.summaryBox.textContent = "选择会议后可生成摘要预览。";
}

function setStatus(text) {
  els.statusText.textContent = text;
}
