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
  materialRows: document.querySelectorAll(".metric-row.clickable"),
  materialListPanel: document.querySelector("#materialListPanel"),
  materialListTitle: document.querySelector("#materialListTitle"),
  materialListItems: document.querySelector("#materialListItems"),
  materialManageBtn: document.querySelector("#materialManageBtn"),
  // 素材管理弹窗
  materialManageOverlay: document.querySelector("#materialManageOverlay"),
  materialManageTitle: document.querySelector("#materialManageTitle"),
  materialManageClose: document.querySelector("#materialManageClose"),
  materialSelectAll: document.querySelector("#materialSelectAll"),
  materialSelectedCount: document.querySelector("#materialSelectedCount"),
  materialManageGrid: document.querySelector("#materialManageGrid"),
  materialDeleteBtn: document.querySelector("#materialDeleteBtn"),
  materialDeleteAllBtn: document.querySelector("#materialDeleteAllBtn"),
  managePageSize: document.querySelector("#managePageSize"),
  managePrevPage: document.querySelector("#managePrevPage"),
  manageNextPage: document.querySelector("#manageNextPage"),
  managePageNum: document.querySelector("#managePageNum"),
  managePageInfo: document.querySelector("#managePageInfo"),
  // 头像管理弹窗
  avatarManageBtn: document.querySelector("#avatarManageBtn"),
  avatarManageOverlay: document.querySelector("#avatarManageOverlay"),
  avatarManageClose: document.querySelector("#avatarManageClose"),
  avatarManageGrid: document.querySelector("#avatarManageGrid"),
  avatarSelectAll: document.querySelector("#avatarSelectAll"),
  avatarSelectedCount: document.querySelector("#avatarSelectedCount"),
  avatarPageSize: document.querySelector("#avatarPageSize"),
  avatarPrevPage: document.querySelector("#avatarPrevPage"),
  avatarNextPage: document.querySelector("#avatarNextPage"),
  avatarPageNum: document.querySelector("#avatarPageNum"),
  avatarPageInfo: document.querySelector("#avatarPageInfo"),
  avatarDeleteBtn: document.querySelector("#avatarDeleteBtn"),
  avatarDeleteAllBtn: document.querySelector("#avatarDeleteAllBtn"),
  // 会议管理弹窗
  manageSessionsBtn: document.querySelector("#manageSessionsBtn"),
  sessionManageOverlay: document.querySelector("#sessionManageOverlay"),
  sessionManageClose: document.querySelector("#sessionManageClose"),
  sessionManageGrid: document.querySelector("#sessionManageGrid"),
  sessionSelectAll: document.querySelector("#sessionSelectAll"),
  sessionSelectedCount: document.querySelector("#sessionSelectedCount"),
  sessionPageSize: document.querySelector("#sessionPageSize"),
  sessionPrevPage: document.querySelector("#sessionPrevPage"),
  sessionNextPage: document.querySelector("#sessionNextPage"),
  sessionPageNum: document.querySelector("#sessionPageNum"),
  sessionPageInfo: document.querySelector("#sessionPageInfo"),
  sessionDeleteBtn: document.querySelector("#sessionDeleteBtn"),
  sessionDeleteAllBtn: document.querySelector("#sessionDeleteAllBtn"),
  summaryBox: document.querySelector("#summaryBox"),
  statusText: document.querySelector("#statusText"),
  taskList: document.querySelector("#taskList"),
  transcriptPanel: document.querySelector("#transcriptPanel"),
  transcriptLive: document.querySelector("#transcriptLive"),
  screenshotViewer: document.querySelector("#screenshotViewer"),
  viewerOverlay: document.querySelector("#viewerOverlay"),
  viewerImg: document.querySelector("#viewerImg"),
  viewerClose: document.querySelector("#viewerClose"),
  viewerPrev: document.querySelector("#viewerPrev"),
  viewerNext: document.querySelector("#viewerNext"),
  viewerInfo: document.querySelector("#viewerInfo"),
  viewerPath: document.querySelector("#viewerPath"),
  // 媒体播放器
  mediaPlayerOverlay: document.querySelector("#mediaPlayerOverlay"),
  mediaPlayerTitle: document.querySelector("#mediaPlayerTitle"),
  mediaPlayerClose: document.querySelector("#mediaPlayerClose"),
  mediaPlayerBody: document.querySelector("#mediaPlayerBody"),
  mediaPlayerPath: document.querySelector("#mediaPlayerPath"),
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
  registerFormUsername: document.querySelector("#registerFormUsername"),
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
  regUsername: document.querySelector("#regUsername"),
  regUsernamePassword: document.querySelector("#regUsernamePassword"),
  regUsernamePasswordConfirm: document.querySelector("#regUsernamePasswordConfirm"),
  registerUsernameBtn: document.querySelector("#registerUsernameBtn"),
  skipLogin: document.querySelector("#skipLogin"),
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
  mainShell: document.querySelector("#mainShell"),
  // 滑块验证码
  sliderCaptcha: document.querySelector("#sliderCaptcha"),
  sliderCaptchaClose: document.querySelector("#sliderCaptchaClose"),
  captchaCanvas: document.querySelector("#captchaCanvas"),
  captchaTrack: document.querySelector("#captchaTrack"),
  captchaTrackText: document.querySelector("#captchaTrackText"),
  captchaThumb: document.querySelector("#captchaThumb"),
  captchaFill: document.querySelector("#captchaFill"),
  // 验证码 Toast
  codeToast: document.querySelector("#codeToast"),
  codeToastValue: document.querySelector("#codeToastValue"),
  // 个人中心
  profileBtn: document.querySelector("#profileBtn"),
  profileOverlay: document.querySelector("#profileOverlay"),
  profileClose: document.querySelector("#profileClose"),
  profileAvatar: document.querySelector("#profileAvatar"),
  profileName: document.querySelector("#profileName"),
  profileUsername: document.querySelector("#profileUsername"),
  profileEmail: document.querySelector("#profileEmail"),
  profilePhone: document.querySelector("#profilePhone"),
  profileApiKey: document.querySelector("#profileApiKey"),
  profileSaveKey: document.querySelector("#profileSaveKey"),
  logoutBtn: document.querySelector("#logoutBtn"),
  changeAvatarBtn: document.querySelector("#changeAvatarBtn"),
  avatarPicker: document.querySelector("#avatarPicker"),
  avatarGrid: document.querySelector("#avatarGrid"),
  avatarUploadBtn: document.querySelector("#avatarUploadBtn"),
  avatarFileInput: document.querySelector("#avatarFileInput")
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
    els.loginFormPhone.hidden = target !== "phone";
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
    els.registerFormUsername.hidden = target !== "username";
  });
});

// 登录/注册模式切换
els.goRegister.addEventListener("click", () => {
  els.loginMode.hidden = true;
  els.registerMode.hidden = false;
});
els.goLogin.addEventListener("click", () => {
  els.registerMode.hidden = true;
  els.loginMode.hidden = false;
});

// 跳过登录
els.skipLogin.addEventListener("click", () => {
  localStorage.setItem("ai_listen_login", JSON.stringify({ method: "skip", info: { username: "游客" }, time: Date.now() }));
  els.loginOverlay.hidden = true;
  els.mainShell.hidden = false;
  setStatus("已跳过登录（游客模式）");
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
  // 同步数据库中的头像到 localStorage（格式一致：{"type":"system",...} 或 {"type":"custom","dataUrl":...}）
  if (info && info.user && info.user.avatar) {
    try {
      const av = JSON.parse(info.user.avatar);
      if (av.type === "system" || av.type === "custom") {
        localStorage.setItem("ai_listen_avatar", info.user.avatar);
      }
    } catch { /* 非 JSON 格式的旧数据，忽略 */ }
  }
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

  // 显示滑块验证码
  showSliderCaptcha(async (success) => {
    if (!success) return;
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
});

els.loginPhoneBtn.addEventListener("click", async () => {
  const phone = els.loginPhone.value.trim();
  const code = els.loginPhoneCode.value.trim();
  if (!phone || !code) { setStatus("请填写手机号和验证码"); return; }

  // 显示滑块验证码
  showSliderCaptcha(async (success) => {
    if (!success) return;
    setStatus("正在登录...");
    try {
      const result = await invoke("login_phone_code", { phone, code });
      if (result.success) {
        doLogin("phone", { phone, user: result.user });
      } else {
        setStatus(result.message);
      }
    } catch (error) {
      setStatus("登录失败：" + String(error));
    }
  });
});

els.loginPhoneSendCode.addEventListener("click", async () => {
  const phone = els.loginPhone.value.trim();
  if (!phone) { setStatus("请先输入手机号"); return; }
  setStatus("正在发送验证码...");
  try {
    const result = await invoke("send_phone_code", { phone });
    showCodeToast(result);
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

  // 显示滑块验证码
  showSliderCaptcha(async (success) => {
    if (!success) return;
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
});

document.querySelectorAll(".social-btn").forEach((btn) => {
  btn.addEventListener("click", async () => {
    const method = btn.dataset.method;

    // 显示滑块验证码
    showSliderCaptcha(async (success) => {
      if (!success) return;
      setStatus(`正在跳转到${btn.textContent}授权...`);
      try {
        const result = await invoke("social_login", { provider: method });
        if (result.success) {
          doLogin(method, { provider: method, user: result.user });
          setStatus(result.message);
        } else {
          setStatus(result.message);
        }
      } catch (error) {
        const errMsg = String(error);
        if (errMsg.includes("未配置")) {
          setStatus(`${btn.textContent}登录未配置：请在 src-tauri/.env 中填入对应平台的密钥`);
        } else {
          setStatus(`${btn.textContent}登录失败：${errMsg}`);
        }
      }
    });
  });
});

// ========== 注册逻辑 ==========
els.regEmailSendCode.addEventListener("click", async () => {
  const email = els.regEmail.value.trim() + els.regEmailProvider.value;
  if (!els.regEmail.value.trim()) { setStatus("请输入邮箱账号"); return; }
  setStatus("正在发送验证码...");
  try {
    const result = await invoke("send_email_code", { email });
    showCodeToast(result);
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

  // 显示滑块验证码
  showSliderCaptcha(async (success) => {
    if (!success) return;
    setStatus("正在验证...");
    try {
      // 后端会校验验证码 + 检查数据库是否已注册
      const result = await invoke("register_email", { email, password, code });
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
});

els.regPhoneSendCode.addEventListener("click", async () => {
  const phone = els.regPhone.value.trim();
  if (!phone) { setStatus("请输入手机号"); return; }
  setStatus("正在发送验证码...");
  try {
    const result = await invoke("send_phone_code", { phone });
    showCodeToast(result);
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

  // 显示滑块验证码
  showSliderCaptcha(async (success) => {
    if (!success) return;
    setStatus("正在验证...");
    try {
      // 后端会校验验证码 + 检查数据库是否已注册
      const result = await invoke("register_phone", { phone, password, code });
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
});

els.registerUsernameBtn.addEventListener("click", async () => {
  const username = els.regUsername.value.trim();
  const password = els.regUsernamePassword.value;
  const confirm = els.regUsernamePasswordConfirm.value;
  if (!username) { setStatus("请输入用户名"); return; }
  if (username.length < 2 || username.length > 20) { setStatus("用户名为2-20个字符"); return; }
  if (password.length < 6 || password.length > 20) { setStatus("密码长度为6-20位"); return; }
  if (password !== confirm) { setStatus("两次密码不一致"); return; }

  // 显示滑块验证码
  showSliderCaptcha(async (success) => {
    if (!success) return;
    setStatus("正在注册...");
    try {
      const result = await invoke("register_username", { username, password });
      if (result.success) {
        localStorage.setItem("ai_listen_login", JSON.stringify({ method: "register_username", info: { username }, user: result.user, time: Date.now() }));
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

// ========== 个人中心逻辑 ==========
els.profileBtn.addEventListener("click", async () => {
  els.profileOverlay.hidden = false;
  // 填充用户信息
  const saved = localStorage.getItem("ai_listen_login");
  if (saved) {
    try {
      const data = JSON.parse(saved);
      const user = data.user || (data.info && data.info.user) || {};
      const name = user.username || (data.info && data.info.username) || "游客";
      els.profileName.textContent = name;
      els.profileUsername.value = name;
      els.profileEmail.value = user.email || "";
      els.profilePhone.value = user.phone || "";
      // 先设置首字母作为默认，再应用已保存的头像（有则覆盖）
      els.profileAvatar.textContent = name.charAt(0).toUpperCase();
      els.profileAvatar.style.background = "";
      els.profileAvatar.style.backgroundImage = "";
      applyAvatar();
    } catch { /* ignore */ }
  }
  // 加载当前 API Key
  try {
    const s = await invoke("load_settings");
    els.profileApiKey.value = s.openai_api_key || "";
  } catch { /* ignore */ }
});
els.profileClose.addEventListener("click", () => { els.profileOverlay.hidden = true; });
els.profileOverlay.addEventListener("click", (e) => {
  if (e.target === els.profileOverlay) els.profileOverlay.hidden = true;
});

// API Key 失焦时自动保存
els.profileApiKey.addEventListener("blur", async () => {
  try {
    const s = await invoke("load_settings");
    await invoke("save_settings_cmd", {
      key: els.profileApiKey.value,
      contextWindow: s.context_window || 4096,
      tokenLimit: s.token_limit || 100000
    });
  } catch { /* ignore */ }
});
els.profileSaveKey.addEventListener("click", async () => {
  try {
    // 保存个人信息到后端数据库
    const saved = localStorage.getItem("ai_listen_login");
    if (saved) {
      const data = JSON.parse(saved);
      // 登录时 user 在 data.info.user，注册时在 data.user
      const user = data.user || (data.info && data.info.user) || {};
      if (user.id) {
        // 读取当前头像（来自 localStorage）
        let avatarValue = null;
        const avatarRaw = localStorage.getItem("ai_listen_avatar");
        let customDataUrl = null;
        let systemEmojiJson = null;
        if (avatarRaw) {
          try {
            const av = JSON.parse(avatarRaw);
            if (av.type === "system") {
              systemEmojiJson = JSON.stringify(av);
              avatarValue = systemEmojiJson;
            } else if (av.type === "custom" && av.dataUrl) {
              customDataUrl = av.dataUrl;
            }
          } catch { /* ignore */ }
        }

        // 如果是系统表情头像，调用 record_system_avatar_cmd 写入 user_avatars 表（is_system=1）
        if (systemEmojiJson) {
          try {
            await invoke("record_system_avatar_cmd", {
              userId: user.id,
              emojiJson: systemEmojiJson
            });
          } catch (sysErr) {
            console.warn("记录系统头像失败:", sysErr);
          }
        }

        // 如果是自定义上传头像，调用 record_custom_avatar_cmd 以 JSON 格式写入 user_avatars 表（与系统头像格式一致）
        if (customDataUrl) {
          const customJson = JSON.stringify({ type: "custom", dataUrl: customDataUrl });
          try {
            await invoke("record_custom_avatar_cmd", {
              userId: user.id,
              avatarJson: customJson
            });
          } catch (uploadErr) {
            console.warn("记录自定义头像失败:", uploadErr);
          }
          avatarValue = customJson;
        }

        const updated = await invoke("update_profile", {
          userId: user.id,
          username: els.profileUsername.value || null,
          email: els.profileEmail.value || null,
          phone: els.profilePhone.value || null,
          avatar: avatarValue
        });
        // 同步更新 localStorage
        if (data.user) { data.user = updated; }
        if (data.info && data.info.user) { data.info.user = updated; }
        localStorage.setItem("ai_listen_login", JSON.stringify(data));
        els.profileName.textContent = updated.username || "用户";
        // 刷新头像显示
        applyAvatar();
        setStatus("保存成功");
        // 保存成功后关闭个人中心弹窗
        els.profileOverlay.hidden = true;
      } else {
        setStatus("保存失败：未找到用户ID，请重新登录");
      }
    } else {
      setStatus("保存失败：未登录");
    }
  } catch (error) {
    setStatus("保存失败：" + String(error));
  }
});
els.logoutBtn.addEventListener("click", () => {
  localStorage.removeItem("ai_listen_login");
  els.profileOverlay.hidden = true;
  els.mainShell.hidden = true;
  els.loginOverlay.hidden = false;
  setStatus("已退出登录");
});

// 头像选择逻辑
const systemAvatars = [
  { emoji: "😀", bg: "#4caf50" },
  { emoji: "🦉", bg: "#2196f3" },
  { emoji: "🚀", bg: "#9c27b0" },
  { emoji: "🎧", bg: "#ff9800" },
  { emoji: "🌟", bg: "#f44336" },
  { emoji: "🐱", bg: "#00bcd4" },
  { emoji: "🌈", bg: "#e91e63" },
  { emoji: "🎯", bg: "#607d8b" },
];

function initAvatarGrid() {
  els.avatarGrid.innerHTML = "";
  systemAvatars.forEach((av) => {
    const btn = document.createElement("button");
    btn.className = "avatar-option";
    btn.style.background = av.bg;
    btn.textContent = av.emoji;
    btn.addEventListener("click", () => {
      saveAvatar({ type: "system", emoji: av.emoji, bg: av.bg });
    });
    els.avatarGrid.appendChild(btn);
  });
}

function saveAvatar(data) {
  localStorage.setItem("ai_listen_avatar", JSON.stringify(data));
  applyAvatar();
  els.avatarPicker.hidden = true;
  setStatus("头像已更新");
}

function applyAvatar() {
  const saved = localStorage.getItem("ai_listen_avatar");
  if (saved) {
    try {
      const data = JSON.parse(saved);
      if (data.type === "system") {
        els.profileAvatar.textContent = data.emoji;
        els.profileAvatar.style.background = data.bg;
        els.profileAvatar.style.backgroundImage = "";
      } else if (data.type === "custom" && data.dataUrl) {
        els.profileAvatar.textContent = "";
        els.profileAvatar.style.background = `url(${data.dataUrl}) center/cover`;
      }
    } catch { /* ignore */ }
  }
}

els.changeAvatarBtn.addEventListener("click", () => {
  els.avatarPicker.hidden = !els.avatarPicker.hidden;
  if (!els.avatarPicker.hidden) initAvatarGrid();
});

els.avatarUploadBtn.addEventListener("click", () => {
  els.avatarFileInput.click();
});

els.avatarFileInput.addEventListener("change", (e) => {
  const file = e.target.files[0];
  if (!file) return;
  // 压缩图片：最大 256x256，JPEG 质量 0.85，避免数据过大导致应用崩溃
  const img = new Image();
  const url = URL.createObjectURL(file);
  img.onload = () => {
    URL.revokeObjectURL(url);
    const MAX = 256;
    let w = img.width, h = img.height;
    if (w > MAX || h > MAX) {
      const scale = Math.min(MAX / w, MAX / h);
      w = Math.round(w * scale);
      h = Math.round(h * scale);
    }
    const canvas = document.createElement("canvas");
    canvas.width = w;
    canvas.height = h;
    const ctx = canvas.getContext("2d");
    ctx.drawImage(img, 0, 0, w, h);
    const dataUrl = canvas.toDataURL("image/jpeg", 0.85);
    saveAvatar({ type: "custom", dataUrl });
  };
  img.onerror = () => {
    URL.revokeObjectURL(url);
    setStatus("图片加载失败");
  };
  img.src = url;
  e.target.value = "";
});

// ========== 素材列表交互逻辑 ==========
let currentMaterialType = null;
let currentSessionData = null;
let manageItems = []; // 管理弹窗中的文件列表

const materialTypeNames = {
  screenshots: "截图",
  recordings: "录屏",
  audio: "音频",
  transcripts: "转写"
};

// 点击素材行展开对应列表
els.materialRows.forEach((row) => {
  row.addEventListener("click", () => {
    const type = row.dataset.type;
    if (currentMaterialType === type && !els.materialListPanel.hidden) {
      els.materialListPanel.hidden = true;
      currentMaterialType = null;
      return;
    }
    currentMaterialType = type;
    els.materialRows.forEach((r) => r.classList.remove("active"));
    row.classList.add("active");
    renderMaterialList();
  });
});

function renderMaterialList() {
  if (!currentSessionData || !currentMaterialType) return;
  const items = currentSessionData[currentMaterialType] || [];
  els.materialListTitle.textContent = `${materialTypeNames[currentMaterialType]} (${items.length})`;
  els.materialListPanel.hidden = false;
  els.materialListItems.innerHTML = "";

  if (items.length === 0) {
    els.materialListItems.innerHTML = '<div class="material-empty">暂无文件</div>';
    return;
  }

  const reversed = [...items].reverse();
  reversed.forEach((path) => {
    const fileName = path.split(/[\\/]/).pop();
    const item = document.createElement("div");
    item.className = "material-item";

    if (currentMaterialType === "screenshots") {
      const img = document.createElement("img");
      img.className = "material-item-thumb";
      loadImageAsDataUrl(path).then((url) => { img.src = url; }).catch(() => {});
      img.addEventListener("click", () => openViewer(path));
      item.appendChild(img);
    } else {
      const icon = document.createElement("span");
      icon.className = "material-item-icon";
      icon.textContent = currentMaterialType === "recordings" ? "🎬" : currentMaterialType === "audio" ? "🎙" : "📝";
      item.appendChild(icon);
      // 录屏和音频可点击播放
      if (currentMaterialType === "recordings" || currentMaterialType === "audio") {
        item.classList.add("playable");
        item.addEventListener("click", () => openMediaPlayer(path, fileName));
      }
    }

    const name = document.createElement("span");
    name.className = "material-item-name";
    name.textContent = fileName;
    name.title = path;
    item.appendChild(name);
    els.materialListItems.appendChild(item);
  });
}

// 管理/删除按钮 → 打开管理弹窗
els.materialManageBtn.addEventListener("click", () => {
  if (!currentSessionData || !currentMaterialType) return;
  openMaterialManage();
});

// ========== 媒体播放器逻辑 ==========
async function openMediaPlayer(path, fileName) {
  els.mediaPlayerTitle.textContent = `▶ ${fileName}`;
  els.mediaPlayerPath.textContent = path;
  els.mediaPlayerBody.innerHTML = '<div class="media-loading">加载中...</div>';
  els.mediaPlayerOverlay.hidden = false;

  try {
    const dataUrl = await invoke("load_media_as_data_url", { path });
    const ext = path.split(".").pop().toLowerCase();
    const isVideo = ["mp4", "mov", "mkv", "webm"].includes(ext);

    els.mediaPlayerBody.innerHTML = "";
    if (isVideo) {
      const video = document.createElement("video");
      video.className = "media-video";
      video.src = dataUrl;
      video.controls = true;
      video.autoplay = true;
      els.mediaPlayerBody.appendChild(video);
    } else {
      const audio = document.createElement("audio");
      audio.className = "media-audio";
      audio.src = dataUrl;
      audio.controls = true;
      audio.autoplay = true;
      els.mediaPlayerBody.appendChild(audio);
    }
  } catch (error) {
    els.mediaPlayerBody.innerHTML = `<div class="media-error">播放失败：${error}</div>`;
  }
}

els.mediaPlayerClose.addEventListener("click", () => {
  els.mediaPlayerOverlay.hidden = true;
  els.mediaPlayerBody.innerHTML = "";
});
els.mediaPlayerOverlay.addEventListener("click", (e) => {
  if (e.target === els.mediaPlayerOverlay) {
    els.mediaPlayerOverlay.hidden = true;
    els.mediaPlayerBody.innerHTML = "";
  }
});

function openMaterialManage() {
  const items = currentSessionData[currentMaterialType] || [];
  manageItems = [...items].reverse();
  manageCurrentPage = 1;
  manageSelectedSet = new Set();
  els.materialManageTitle.textContent = `🗑 管理${materialTypeNames[currentMaterialType]}`;
  els.materialManageOverlay.hidden = false;
  els.materialSelectAll.checked = false;
  renderManageGrid();
}

let manageCurrentPage = 1;
let manageSelectedSet = new Set(); // 存储选中的文件路径

function getManagePageSize() {
  return parseInt(els.managePageSize.value) || 20;
}

function getManageTotalPages() {
  return Math.max(1, Math.ceil(manageItems.length / getManagePageSize()));
}

function renderManageGrid() {
  const pageSize = getManagePageSize();
  const totalPages = getManageTotalPages();
  if (manageCurrentPage > totalPages) manageCurrentPage = totalPages;

  const start = (manageCurrentPage - 1) * pageSize;
  const pageItems = manageItems.slice(start, start + pageSize);

  els.materialManageGrid.innerHTML = "";
  pageItems.forEach((path, idx) => {
    const globalIdx = start + idx;
    const fileName = path.split(/[\\/]/).pop();
    const card = document.createElement("div");
    card.className = "manage-card";
    card.dataset.index = globalIdx;
    card.dataset.path = path;

    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.className = "manage-card-cb";
    cb.checked = manageSelectedSet.has(path);
    cb.addEventListener("change", () => {
      if (cb.checked) { manageSelectedSet.add(path); } else { manageSelectedSet.delete(path); }
      updateManageCount();
    });
    card.appendChild(cb);

    if (currentMaterialType === "screenshots") {
      const img = document.createElement("img");
      img.className = "manage-card-img";
      loadImageAsDataUrl(path).then((url) => { img.src = url; }).catch(() => {});
      card.appendChild(img);
    } else {
      const icon = document.createElement("div");
      icon.className = "manage-card-icon";
      icon.textContent = currentMaterialType === "recordings" ? "🎬" : currentMaterialType === "audio" ? "🎙" : "📝";
      card.appendChild(icon);
    }

    const label = document.createElement("div");
    label.className = "manage-card-name";
    label.textContent = fileName;
    label.title = path;
    card.appendChild(label);
    els.materialManageGrid.appendChild(card);
  });

  // 更新分页控件
  els.managePageNum.textContent = `${manageCurrentPage}/${totalPages}`;
  els.managePageInfo.textContent = `共 ${manageItems.length} 项`;
  els.managePrevPage.disabled = manageCurrentPage <= 1;
  els.manageNextPage.disabled = manageCurrentPage >= totalPages;
  updateManageCount();
}

function updateManageCount() {
  const checked = manageSelectedSet.size;
  els.materialSelectedCount.textContent = `已选 ${checked} 项`;
  els.materialDeleteBtn.disabled = checked === 0;

  // 全选状态：当前页所有项都选中
  const boxes = els.materialManageGrid.querySelectorAll(".manage-card-cb");
  const pageChecked = els.materialManageGrid.querySelectorAll(".manage-card-cb:checked").length;
  els.materialSelectAll.checked = boxes.length > 0 && pageChecked === boxes.length;
}

// 全选/取消全选（当前页）
els.materialSelectAll.addEventListener("change", () => {
  const cards = els.materialManageGrid.querySelectorAll(".manage-card");
  cards.forEach((card) => {
    const cb = card.querySelector(".manage-card-cb");
    const path = card.dataset.path;
    cb.checked = els.materialSelectAll.checked;
    if (els.materialSelectAll.checked) { manageSelectedSet.add(path); } else { manageSelectedSet.delete(path); }
  });
  updateManageCount();
});

// 分页切换
els.managePrevPage.addEventListener("click", () => {
  if (manageCurrentPage > 1) { manageCurrentPage--; renderManageGrid(); }
});
els.manageNextPage.addEventListener("click", () => {
  if (manageCurrentPage < getManageTotalPages()) { manageCurrentPage++; renderManageGrid(); }
});
els.managePageSize.addEventListener("change", () => {
  manageCurrentPage = 1;
  renderManageGrid();
});

els.materialManageClose.addEventListener("click", () => { els.materialManageOverlay.hidden = true; });
els.materialManageOverlay.addEventListener("click", (e) => {
  if (e.target === els.materialManageOverlay) els.materialManageOverlay.hidden = true;
});

els.materialDeleteBtn.addEventListener("click", async () => {
  const paths = Array.from(manageSelectedSet);
  if (paths.length === 0) return;
  if (!confirm(`确认删除选中的 ${paths.length} 个文件？`)) return;
  try {
    let session;
    for (const p of paths) {
      session = await invoke("delete_material_file", { slug: activeSlug, path: p });
    }
    if (session) {
      currentSessionData = session;
      renderEditor(session);
      renderMaterialList();
    }
    // 刷新管理弹窗列表
    manageItems = manageItems.filter((p) => !manageSelectedSet.has(p));
    manageSelectedSet = new Set();
    els.materialSelectAll.checked = false;
    renderManageGrid();
    setStatus(`已删除 ${paths.length} 个文件`);
    if (manageItems.length === 0) {
      els.materialManageOverlay.hidden = true;
    }
  } catch (error) {
    setStatus("删除失败：" + String(error));
  }
});

// 全部删除（与删除选中相同的交互逻辑：确认 → 删除 → 刷新）
els.materialDeleteAllBtn.addEventListener("click", async () => {
  const total = manageItems.length;
  if (total === 0) return;
  if (!confirm(`确认删除全部 ${total} 个${materialTypeNames[currentMaterialType] || "文件"}？`)) return;
  try {
    const session = await invoke("delete_all_material_cmd", { slug: activeSlug, materialType: currentMaterialType });
    if (session) {
      currentSessionData = session;
      renderEditor(session);
      renderMaterialList();
    }
    // 刷新管理弹窗列表
    manageItems = [];
    manageSelectedSet = new Set();
    els.materialSelectAll.checked = false;
    renderManageGrid();
    setStatus(`已删除全部 ${total} 个文件`);
    els.materialManageOverlay.hidden = true;
  } catch (error) {
    setStatus("删除失败：" + String(error));
  }
});

// ========== 会议管理逻辑 ==========
let sessionManageItems = []; // 全部会议列表
let sessionManagePage = 1;
let sessionSelectedSet = new Set(); // 选中的 slug

async function openSessionManage() {
  try {
    sessionManageItems = await invoke("list_sessions");
  } catch (e) {
    setStatus("加载会议列表失败: " + String(e));
    return;
  }
  sessionManagePage = 1;
  sessionSelectedSet = new Set();
  els.sessionSelectAll.checked = false;
  els.sessionManageOverlay.hidden = false;
  renderSessionManageGrid();
}

function getSessionPageSize() {
  return parseInt(els.sessionPageSize.value) || 20;
}

function getSessionTotalPages() {
  return Math.max(1, Math.ceil(sessionManageItems.length / getSessionPageSize()));
}

function renderSessionManageGrid() {
  const pageSize = getSessionPageSize();
  const totalPages = getSessionTotalPages();
  if (sessionManagePage > totalPages) sessionManagePage = totalPages;
  const start = (sessionManagePage - 1) * pageSize;
  const pageItems = sessionManageItems.slice(start, start + pageSize);

  els.sessionManageGrid.innerHTML = "";
  pageItems.forEach((s) => {
    const card = document.createElement("div");
    card.className = "manage-card";

    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.className = "manage-card-cb";
    cb.checked = sessionSelectedSet.has(s.slug);
    cb.addEventListener("change", () => {
      if (cb.checked) { sessionSelectedSet.add(s.slug); } else { sessionSelectedSet.delete(s.slug); }
      updateSessionManageCount();
    });
    card.appendChild(cb);

    const icon = document.createElement("span");
    icon.className = "manage-card-icon";
    icon.textContent = "📝";
    card.appendChild(icon);

    const name = document.createElement("span");
    name.className = "manage-card-name";
    const counts = `截图${(s.screenshots || []).length} 录屏${(s.recordings || []).length} 音频${(s.audio || []).length} 转写${(s.transcripts || []).length}`;
    name.textContent = `${s.title} (${s.slug}) · ${counts}`;
    name.title = s.path || s.slug;
    card.appendChild(name);

    els.sessionManageGrid.appendChild(card);
  });

  els.sessionPageInfo.textContent = `共 ${sessionManageItems.length} 项`;
  els.sessionPageNum.textContent = `${sessionManagePage}/${totalPages}`;
  els.sessionPrevPage.disabled = sessionManagePage <= 1;
  els.sessionNextPage.disabled = sessionManagePage >= totalPages;
  updateSessionManageCount();
}

function updateSessionManageCount() {
  const checked = sessionSelectedSet.size;
  els.sessionSelectedCount.textContent = `已选 ${checked} 项`;
  els.sessionDeleteBtn.disabled = checked === 0;
}

els.manageSessionsBtn.addEventListener("click", () => { openSessionManage(); });
els.sessionManageClose.addEventListener("click", () => { els.sessionManageOverlay.hidden = true; });
els.sessionManageOverlay.addEventListener("click", (e) => {
  if (e.target === els.sessionManageOverlay) els.sessionManageOverlay.hidden = true;
});
els.sessionPrevPage.addEventListener("click", () => {
  if (sessionManagePage > 1) { sessionManagePage--; renderSessionManageGrid(); }
});
els.sessionNextPage.addEventListener("click", () => {
  if (sessionManagePage < getSessionTotalPages()) { sessionManagePage++; renderSessionManageGrid(); }
});
els.sessionPageSize.addEventListener("change", () => {
  sessionManagePage = 1;
  renderSessionManageGrid();
});
els.sessionSelectAll.addEventListener("change", () => {
  const pageSize = getSessionPageSize();
  const start = (sessionManagePage - 1) * pageSize;
  const pageItems = sessionManageItems.slice(start, start + pageSize);
  pageItems.forEach((s) => {
    if (els.sessionSelectAll.checked) { sessionSelectedSet.add(s.slug); } else { sessionSelectedSet.delete(s.slug); }
  });
  renderSessionManageGrid();
});

// 删除选中会议
els.sessionDeleteBtn.addEventListener("click", async () => {
  const slugs = Array.from(sessionSelectedSet);
  if (slugs.length === 0) return;
  if (!confirm(`确认删除选中的 ${slugs.length} 个会议？\n删除后不可恢复（包含所有截图/录音/转写文件）`)) return;
  try {
    await invoke("delete_sessions_cmd", { slugs });
    sessionManageItems = sessionManageItems.filter((s) => !sessionSelectedSet.has(s.slug));
    sessionSelectedSet = new Set();
    els.sessionSelectAll.checked = false;
    renderSessionManageGrid();
    setStatus(`已删除 ${slugs.length} 个会议`);
    // 刷新侧边栏会议列表
    activeSlug = sessionManageItems.length ? sessionManageItems[0].slug : null;
    await loadSessions();
  } catch (e) {
    setStatus("删除失败: " + String(e));
  }
});

// 全部删除会议
els.sessionDeleteAllBtn.addEventListener("click", async () => {
  const total = sessionManageItems.length;
  if (total === 0) return;
  if (!confirm(`确认删除全部 ${total} 个会议？\n删除后不可恢复（包含所有截图/录音/转写文件）`)) return;
  try {
    const slugs = sessionManageItems.map((s) => s.slug);
    await invoke("delete_sessions_cmd", { slugs });
    sessionManageItems = [];
    sessionSelectedSet = new Set();
    els.sessionSelectAll.checked = false;
    renderSessionManageGrid();
    setStatus(`已删除全部 ${total} 个会议`);
    activeSlug = null;
    await loadSessions();
  } catch (e) {
    setStatus("删除失败: " + String(e));
  }
});

// ========== 头像管理逻辑 ==========
let avatarManageItems = []; // 全部头像记录
let avatarManagePage = 1;
let avatarSelectedSet = new Set(); // 选中的头像 ID

function getAvatarUserId() {
  const saved = localStorage.getItem("ai_listen_login");
  if (!saved) return null;
  try {
    const data = JSON.parse(saved);
    const user = data.user || (data.info && data.info.user) || {};
    return user.id || null;
  } catch { return null; }
}

async function openAvatarManage() {
  const userId = getAvatarUserId();
  if (!userId) { setStatus("请先登录"); return; }
  try {
    avatarManageItems = await invoke("list_avatars_cmd", { userId });
  } catch (e) {
    setStatus("加载头像列表失败: " + String(e));
    return;
  }
  avatarManagePage = 1;
  avatarSelectedSet = new Set();
  els.avatarSelectAll.checked = false;
  els.avatarManageOverlay.hidden = false;
  renderAvatarManageGrid();
}

function getAvatarPageSize() {
  return parseInt(els.avatarPageSize.value) || 20;
}

function getAvatarTotalPages() {
  return Math.max(1, Math.ceil(avatarManageItems.length / getAvatarPageSize()));
}

function renderAvatarManageGrid() {
  const pageSize = getAvatarPageSize();
  const totalPages = getAvatarTotalPages();
  if (avatarManagePage > totalPages) avatarManagePage = totalPages;
  const start = (avatarManagePage - 1) * pageSize;
  const pageItems = avatarManageItems.slice(start, start + pageSize);

  els.avatarManageGrid.innerHTML = "";
  pageItems.forEach((av) => {
    const card = document.createElement("div");
    card.className = "manage-card";

    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.className = "manage-card-cb";
    cb.checked = avatarSelectedSet.has(av.id);
    cb.addEventListener("change", () => {
      if (cb.checked) { avatarSelectedSet.add(av.id); } else { avatarSelectedSet.delete(av.id); }
      updateAvatarManageCount();
    });
    card.appendChild(cb);

    // 头像预览
    try {
      const data = JSON.parse(av.file_path);
      if (data.type === "system") {
        const icon = document.createElement("span");
        icon.className = "manage-card-icon";
        icon.textContent = data.emoji || "😀";
        card.appendChild(icon);
      } else if (data.type === "custom" && data.dataUrl) {
        const img = document.createElement("img");
        img.className = "manage-card-img";
        img.src = data.dataUrl;
        card.appendChild(img);
      }
    } catch {
      const icon = document.createElement("span");
      icon.className = "manage-card-icon";
      icon.textContent = "🖼";
      card.appendChild(icon);
    }

    const name = document.createElement("span");
    name.className = "manage-card-name";
    const typeLabel = av.is_system ? "系统" : "自定义";
    const date = av.created_at ? new Date(parseInt(av.created_at) * 1000).toLocaleString() : "";
    name.textContent = `${typeLabel} · ${av.file_name || "avatar"} · ${date}`;
    name.title = av.id;
    card.appendChild(name);

    // 使用按钮：将该头像设为当前头像
    const useBtn = document.createElement("button");
    useBtn.className = "btn-danger-sm";
    useBtn.style.cssText = "background:#4caf50;border:none;color:#fff;cursor:pointer;flex-shrink:0;";
    useBtn.textContent = "使用";
    useBtn.addEventListener("click", async () => {
      const userId = getAvatarUserId();
      if (!userId) return;
      try {
        const updated = await invoke("apply_avatar_cmd", { userId, avatarJson: av.file_path });
        // 同步到 localStorage
        localStorage.setItem("ai_listen_avatar", av.file_path);
        const savedLogin = localStorage.getItem("ai_listen_login");
        if (savedLogin) {
          const loginData = JSON.parse(savedLogin);
          if (loginData.user) loginData.user = updated;
          if (loginData.info && loginData.info.user) loginData.info.user = updated;
          localStorage.setItem("ai_listen_login", JSON.stringify(loginData));
        }
        applyAvatar();
        setStatus("头像已更新");
      } catch (e) {
        setStatus("设置头像失败: " + String(e));
      }
    });
    card.appendChild(useBtn);

    els.avatarManageGrid.appendChild(card);
  });

  els.avatarPageInfo.textContent = `共 ${avatarManageItems.length} 项`;
  els.avatarPageNum.textContent = `${avatarManagePage}/${totalPages}`;
  els.avatarPrevPage.disabled = avatarManagePage <= 1;
  els.avatarNextPage.disabled = avatarManagePage >= totalPages;
  updateAvatarManageCount();
}

function updateAvatarManageCount() {
  const checked = avatarSelectedSet.size;
  els.avatarSelectedCount.textContent = `已选 ${checked} 项`;
  els.avatarDeleteBtn.disabled = checked === 0;
}

els.avatarManageBtn.addEventListener("click", () => { openAvatarManage(); });
els.avatarManageClose.addEventListener("click", () => { els.avatarManageOverlay.hidden = true; });
els.avatarManageOverlay.addEventListener("click", (e) => {
  if (e.target === els.avatarManageOverlay) els.avatarManageOverlay.hidden = true;
});
els.avatarPrevPage.addEventListener("click", () => {
  if (avatarManagePage > 1) { avatarManagePage--; renderAvatarManageGrid(); }
});
els.avatarNextPage.addEventListener("click", () => {
  if (avatarManagePage < getAvatarTotalPages()) { avatarManagePage++; renderAvatarManageGrid(); }
});
els.avatarPageSize.addEventListener("change", () => {
  avatarManagePage = 1;
  renderAvatarManageGrid();
});
els.avatarSelectAll.addEventListener("change", () => {
  const pageSize = getAvatarPageSize();
  const start = (avatarManagePage - 1) * pageSize;
  const pageItems = avatarManageItems.slice(start, start + pageSize);
  pageItems.forEach((av) => {
    if (els.avatarSelectAll.checked) { avatarSelectedSet.add(av.id); } else { avatarSelectedSet.delete(av.id); }
  });
  renderAvatarManageGrid();
});

// 删除选中头像记录
els.avatarDeleteBtn.addEventListener("click", async () => {
  const ids = Array.from(avatarSelectedSet);
  if (ids.length === 0) return;
  if (!confirm(`确认删除选中的 ${ids.length} 条头像记录？`)) return;
  const userId = getAvatarUserId();
  if (!userId) return;
  try {
    await invoke("delete_avatars_cmd", { userId, avatarIds: ids });
    avatarManageItems = avatarManageItems.filter((av) => !avatarSelectedSet.has(av.id));
    avatarSelectedSet = new Set();
    els.avatarSelectAll.checked = false;
    renderAvatarManageGrid();
    setStatus(`已删除 ${ids.length} 条头像记录`);
  } catch (e) {
    setStatus("删除失败: " + String(e));
  }
});

// 全部删除头像记录
els.avatarDeleteAllBtn.addEventListener("click", async () => {
  const total = avatarManageItems.length;
  if (total === 0) return;
  if (!confirm(`确认删除全部 ${total} 条头像记录？`)) return;
  const userId = getAvatarUserId();
  if (!userId) return;
  try {
    const ids = avatarManageItems.map((av) => av.id);
    await invoke("delete_avatars_cmd", { userId, avatarIds: ids });
    avatarManageItems = [];
    avatarSelectedSet = new Set();
    els.avatarSelectAll.checked = false;
    renderAvatarManageGrid();
    setStatus(`已删除全部 ${total} 条头像记录`);
  } catch (e) {
    setStatus("删除失败: " + String(e));
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
  currentSessionData = session;
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
  // 更新截图查看器数据
  viewerScreenshots = session.screenshots;
  // 刷新当前展开的素材列表
  if (currentMaterialType) renderMaterialList();
}

// 截图查看器
let viewerScreenshots = [];
let viewerCurrentIndex = 0;

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
  els.viewerPath.textContent = path;
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

// 验证码 Toast 提示（开发模式下显示验证码）
let codeToastTimer = null;
function showCodeToast(message) {
  // 从后端返回的消息中提取验证码
  const match = message.match(/验证码：(\d{6})/);
  if (match) {
    els.codeToastValue.textContent = match[1];
    els.codeToast.hidden = false;
    clearTimeout(codeToastTimer);
    codeToastTimer = setTimeout(() => { els.codeToast.hidden = true; }, 30000);
  }
  setStatus(message);
}

// ========== 滑块验证码 ==========
let captchaCallback = null;
let captchaTargetX = 0;
let captchaTargetY = 0;
let captchaIsDragging = false;
let captchaStartX = 0;
let captchaBgShapes = []; // 保存背景图形用于重绘

function drawCaptchaCanvas(pieceX) {
  const canvas = els.captchaCanvas;
  const ctx = canvas.getContext("2d");
  const width = canvas.width;
  const height = canvas.height;

  // 清空画布
  ctx.clearRect(0, 0, width, height);

  // 绘制背景渐变
  const gradient = ctx.createLinearGradient(0, 0, width, height);
  gradient.addColorStop(0, "#e8f5e9");
  gradient.addColorStop(0.5, "#f0f5f2");
  gradient.addColorStop(1, "#e0f2f1");
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, width, height);

  // 绘制保存的背景图形
  captchaBgShapes.forEach((s) => {
    ctx.beginPath();
    ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2);
    ctx.fillStyle = s.color;
    ctx.fill();
  });

  // 绘制目标凹槽（固定位置）
  ctx.beginPath();
  ctx.arc(captchaTargetX, captchaTargetY, 20, 0, Math.PI * 2);
  ctx.fillStyle = "rgba(0, 0, 0, 0.15)";
  ctx.fill();
  ctx.strokeStyle = "rgba(0, 0, 0, 0.3)";
  ctx.lineWidth = 2;
  ctx.setLineDash([4, 3]);
  ctx.stroke();
  ctx.setLineDash([]);

  // 绘制可移动的拼图块（跟随滑块）
  ctx.beginPath();
  ctx.arc(pieceX, captchaTargetY, 18, 0, Math.PI * 2);
  ctx.fillStyle = "#2e7d5b";
  ctx.shadowColor = "rgba(0,0,0,0.3)";
  ctx.shadowBlur = 6;
  ctx.shadowOffsetX = 2;
  ctx.shadowOffsetY = 2;
  ctx.fill();
  ctx.shadowColor = "transparent";
  ctx.shadowBlur = 0;
  ctx.shadowOffsetX = 0;
  ctx.shadowOffsetY = 0;
  ctx.strokeStyle = "#1b5e40";
  ctx.lineWidth = 2;
  ctx.stroke();
}

function initSliderCaptcha() {
  const canvas = els.captchaCanvas;
  const width = canvas.width;
  const height = canvas.height;

  // 生成随机背景图形
  captchaBgShapes = [];
  for (let i = 0; i < 8; i++) {
    captchaBgShapes.push({
      x: Math.random() * width,
      y: Math.random() * height,
      r: Math.random() * 30 + 10,
      color: `rgba(${Math.random() * 100 + 100}, ${Math.random() * 150 + 100}, ${Math.random() * 100 + 100}, 0.3)`,
    });
  }

  // 目标位置
  captchaTargetX = Math.random() * (width - 100) + 60;
  captchaTargetY = Math.random() * (height - 60) + 30;

  // 初始绘制（拼图块在最左侧）
  drawCaptchaCanvas(20);

  // 重置滑块位置
  els.captchaThumb.style.left = "2px";
  els.captchaFill.style.width = "0px";
  els.captchaFill.style.background = "";
  els.captchaThumb.className = "slider-captcha-thumb";
  els.captchaThumb.innerHTML = "&rarr;";
  els.captchaTrackText.textContent = "拖动滑块完成验证";
}

function showSliderCaptcha(callback) {
  captchaCallback = callback;
  els.sliderCaptcha.hidden = false;
  initSliderCaptcha();
}

function hideSliderCaptcha() {
  els.sliderCaptcha.hidden = true;
  captchaCallback = null;
}

// 关闭按钮
els.sliderCaptchaClose.addEventListener("click", hideSliderCaptcha);
els.sliderCaptcha.addEventListener("click", (e) => {
  if (e.target === els.sliderCaptcha) hideSliderCaptcha();
});

// 滑块拖拽逻辑
els.captchaThumb.addEventListener("mousedown", (e) => {
  captchaIsDragging = true;
  captchaStartX = e.clientX;
  e.preventDefault();
});

document.addEventListener("mousemove", (e) => {
  if (!captchaIsDragging) return;

  const trackWidth = els.captchaTrack.offsetWidth;
  const thumbWidth = els.captchaThumb.offsetWidth;
  const maxMove = trackWidth - thumbWidth - 4;

  let deltaX = e.clientX - captchaStartX;
  deltaX = Math.max(0, Math.min(deltaX, maxMove));

  els.captchaThumb.style.left = `${deltaX + 2}px`;
  els.captchaFill.style.width = `${deltaX + thumbWidth / 2}px`;

  // 将滑块位移映射到 canvas 拼图块的 x 坐标
  const canvasWidth = els.captchaCanvas.width;
  const pieceX = 20 + (deltaX / maxMove) * (canvasWidth - 40);
  drawCaptchaCanvas(pieceX);
});

document.addEventListener("mouseup", () => {
  if (!captchaIsDragging) return;
  captchaIsDragging = false;

  const thumbLeft = parseInt(els.captchaThumb.style.left) || 2;
  // 将滑块位置映射到 canvas 坐标进行比较
  const trackWidth = els.captchaTrack.offsetWidth;
  const thumbWidth = els.captchaThumb.offsetWidth;
  const maxMove = trackWidth - thumbWidth - 4;
  const canvasWidth = els.captchaCanvas.width;
  const pieceX = 20 + ((thumbLeft - 2) / maxMove) * (canvasWidth - 40);
  const tolerance = 12; // 允许误差

  if (Math.abs(pieceX - captchaTargetX) < tolerance) {
    // 验证成功
    els.captchaThumb.className = "slider-captcha-thumb success";
    els.captchaThumb.innerHTML = "&check;";
    els.captchaTrackText.textContent = "验证成功";
    els.captchaFill.style.background = "#07c160";

    setTimeout(() => {
      const cb = captchaCallback;
      hideSliderCaptcha();
      if (cb) cb(true);
    }, 500);
  } else {
    // 验证失败
    els.captchaThumb.className = "slider-captcha-thumb fail";
    els.captchaThumb.innerHTML = "&times;";
    els.captchaTrackText.textContent = "验证失败，请重试";
    els.captchaFill.style.background = "#ff4d4f";

    setTimeout(() => {
      initSliderCaptcha();
    }, 1000);
  }
});
