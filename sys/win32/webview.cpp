#include "webview.hpp"

WebView::WebView(HWND _hWndParent) {
	if (webview_ie_compat_mode(11000) < 0) {
		return;
	}

	iComRefCount = 0;
	RECT rect;
	GetClientRect(_hWndParent, &rect);
	::SetRect(&rObject, rect.left, rect.top, rect.right, rect.bottom);
	hWndParent = _hWndParent;

	if (CreateBrowser() == FALSE) {
		return;
	}

	ShowWindow(GetControlWindow(), SW_SHOW);

	wchar_t url[] = L"about:blank\0";
	this->Navigate(url);
}

bool WebView::CreateBrowser() {
	HRESULT hr;
	hr = ::OleCreate(CLSID_WebBrowser, IID_IOleObject, OLERENDER_DRAW, 0, this,
			this, (void**) &oleObject);

	if (FAILED(hr)) {
		MessageBox(NULL, _T("Cannot create oleObject CLSID_WebBrowser"),
				_T("Error"), MB_ICONERROR);
		return FALSE;
	}

	hr = oleObject->SetClientSite(this);
	hr = OleSetContainedObject(oleObject, TRUE);

	RECT rect;
	GetClientRect(hWndParent, &rect);
	RECT posRect;
	::SetRect(&posRect, rect.left, rect.top, rect.right, rect.bottom);
	hr = oleObject->DoVerb(OLEIVERB_INPLACEACTIVATE, NULL, this, -1, hWndParent,
			&posRect);
	if (FAILED(hr)) {
		MessageBox(NULL, _T("oleObject->DoVerb() failed"), _T("Error"),
				MB_ICONERROR);
		return FALSE;
	}

	hr = oleObject->QueryInterface(&webBrowser2);
	if (FAILED(hr)) {
		MessageBox(NULL, _T("oleObject->QueryInterface(&webBrowser2) failed"),
				_T("Error"), MB_ICONERROR);
		return FALSE;
	}

	return TRUE;
}

RECT WebView::PixelToHiMetric(const RECT& _rc) {
	static bool s_initialized = false;
	static int s_pixelsPerInchX, s_pixelsPerInchY;
	if (!s_initialized) {
		HDC hdc = ::GetDC(0);
		s_pixelsPerInchX = ::GetDeviceCaps(hdc, LOGPIXELSX);
		s_pixelsPerInchY = ::GetDeviceCaps(hdc, LOGPIXELSY);
		::ReleaseDC(0, hdc);
		s_initialized = true;
	}

	RECT rc;
	rc.left = MulDiv(2540, _rc.left, s_pixelsPerInchX);
	rc.top = MulDiv(2540, _rc.top, s_pixelsPerInchY);
	rc.right = MulDiv(2540, _rc.right, s_pixelsPerInchX);
	rc.bottom = MulDiv(2540, _rc.bottom, s_pixelsPerInchY);
	return rc;
}

void WebView::SetRect(const RECT& _rc) {
	rObject = _rc;

	{
		RECT hiMetricRect = PixelToHiMetric(rObject);
		SIZEL sz;
		sz.cx = hiMetricRect.right - hiMetricRect.left;
		sz.cy = hiMetricRect.bottom - hiMetricRect.top;
		oleObject->SetExtent(DVASPECT_CONTENT, &sz);
	}

	if (oleInPlaceObject != 0) {
		oleInPlaceObject->SetObjectRects(&rObject, &rObject);
	}
}

// ----- Control methods -----

void WebView::GoBack() {
	this->webBrowser2->GoBack();
}

void WebView::GoForward() {
	this->webBrowser2->GoForward();
}

void WebView::Refresh() {
	this->webBrowser2->Refresh();
}

void WebView::Navigate(wchar_t* szUrl) {
	variant_t flags(0x02u);
	this->webBrowser2->Navigate(szUrl, &flags, 0, 0, 0);
}

HRESULT WebView::LocationURL(wchar_t** pszUrl) {
	return this->webBrowser2->get_LocationURL(pszUrl);
}

// ----- IUnknown -----

HRESULT STDMETHODCALLTYPE WebView::QueryInterface(REFIID riid,
		void**ppvObject) {
	if (riid == __uuidof(IUnknown)) {
		(*ppvObject) = static_cast<IOleClientSite*>(this);
	} else if (riid == __uuidof(IOleInPlaceSite)) {
		(*ppvObject) = static_cast<IOleInPlaceSite*>(this);
	} else if (riid == __uuidof(IOleCommandTarget)) {
		(*ppvObject) = static_cast<IOleCommandTarget*>(this);
	} else {
		return E_NOINTERFACE;
	}

	AddRef();
	return S_OK;
}

ULONG STDMETHODCALLTYPE WebView::AddRef(void) {
	iComRefCount++;
	return iComRefCount;
}

ULONG STDMETHODCALLTYPE WebView::Release(void) {
	iComRefCount--;
	return iComRefCount;
}

// ---------- IOleCommandTarget ---

HRESULT STDMETHODCALLTYPE WebView::Exec(const GUID *pguidCmdGroup, DWORD nCmdID,
		DWORD nCmdexecopt, VARIANT *pvaIn, VARIANT *pvaOut) {
	if (nCmdID == OLECMDID_SHOWSCRIPTERROR) {
		(*pvaOut).vt = VT_BOOL;
		(*pvaOut).boolVal = VARIANT_TRUE;
		return S_OK;
	}
	return S_OK;
}

HRESULT STDMETHODCALLTYPE WebView::QueryStatus(const GUID *pguidCmdGroup,
		ULONG cCmds, OLECMD prgCmds[], OLECMDTEXT *pCmdText) {
	return S_OK;
}

// ---------- IOleWindow ----------

HRESULT STDMETHODCALLTYPE WebView::GetWindow(
		__RPC__deref_out_opt HWND *phwnd)
{
	(*phwnd) = hWndParent;
	return S_OK;
}

HRESULT STDMETHODCALLTYPE WebView::ContextSensitiveHelp(BOOL fEnterMode) {
	return E_NOTIMPL;
}

// ---------- IOleInPlaceSite ----------

HRESULT STDMETHODCALLTYPE WebView::CanInPlaceActivate(void) {
	return S_OK;
}

HRESULT STDMETHODCALLTYPE WebView::OnInPlaceActivate(void) {
	OleLockRunning(oleObject, TRUE, FALSE);
	oleObject->QueryInterface(&oleInPlaceObject);
	oleInPlaceObject->SetObjectRects(&rObject, &rObject);

	return S_OK;

}

HRESULT STDMETHODCALLTYPE WebView::OnUIActivate(void) {
	return S_OK;
}

HRESULT STDMETHODCALLTYPE WebView::GetWindowContext(
		__RPC__deref_out_opt IOleInPlaceFrame **ppFrame,
		__RPC__deref_out_opt IOleInPlaceUIWindow **ppDoc,
		__RPC__out LPRECT lprcPosRect,
		__RPC__out LPRECT lprcClipRect,
		__RPC__inout LPOLEINPLACEFRAMEINFO lpFrameInfo)
{
	HWND hwnd = hWndParent;

	(*ppFrame) = NULL;
	(*ppDoc) = NULL;
	(*lprcPosRect).left = rObject.left;
	(*lprcPosRect).top = rObject.top;
	(*lprcPosRect).right = rObject.right;
	(*lprcPosRect).bottom = rObject.bottom;
	*lprcClipRect = *lprcPosRect;

	lpFrameInfo->fMDIApp = false;
	lpFrameInfo->hwndFrame = hwnd;
	lpFrameInfo->haccel = NULL;
	lpFrameInfo->cAccelEntries = 0;

	return S_OK;
}

HRESULT STDMETHODCALLTYPE WebView::Scroll(SIZE scrollExtant) {
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::OnUIDeactivate(BOOL fUndoable) {
	return S_OK;
}

HWND WebView::GetControlWindow() {
	if (hWndControl != 0)
		return hWndControl;

	if (oleInPlaceObject == 0)
		return 0;

	oleInPlaceObject->GetWindow(&hWndControl);
	return hWndControl;
}

HRESULT STDMETHODCALLTYPE WebView::OnInPlaceDeactivate(void) {
	hWndControl = 0;
	oleInPlaceObject = 0;

	return S_OK;
}

HRESULT STDMETHODCALLTYPE WebView::DiscardUndoState(void) {
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::DeactivateAndUndo(void) {
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::OnPosRectChange(
		__RPC__in LPCRECT lprcPosRect)
{
	return E_NOTIMPL;
}

// ---------- IOleClientSite ----------

HRESULT STDMETHODCALLTYPE WebView::SaveObject(void) {
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::GetMoniker(
		DWORD dwAssign,
		DWORD dwWhichMoniker,
		__RPC__deref_out_opt IMoniker **ppmk)
{
	if((dwAssign == OLEGETMONIKER_ONLYIFTHERE) &&
			(dwWhichMoniker == OLEWHICHMK_CONTAINER))
	return E_FAIL;

	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::GetContainer(
		__RPC__deref_out_opt IOleContainer **ppContainer)
{
	return E_NOINTERFACE;
}

HRESULT STDMETHODCALLTYPE WebView::ShowObject(void) {
	return S_OK;
}

HRESULT STDMETHODCALLTYPE WebView::OnShowWindow(BOOL fShow) {
	return S_OK;
}

HRESULT STDMETHODCALLTYPE WebView::RequestNewObjectLayout(void) {
	return E_NOTIMPL;
}

// ----- IStorage -----

HRESULT STDMETHODCALLTYPE WebView::CreateStream(
		__RPC__in_string const OLECHAR *pwcsName,
		DWORD grfMode,
		DWORD reserved1,
		DWORD reserved2,
		__RPC__deref_out_opt IStream **ppstm)
{
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::OpenStream(const OLECHAR *pwcsName,
		void *reserved1, DWORD grfMode, DWORD reserved2, IStream **ppstm) {
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::CreateStorage(
		__RPC__in_string const OLECHAR *pwcsName,
		DWORD grfMode,
		DWORD reserved1,
		DWORD reserved2,
		__RPC__deref_out_opt IStorage **ppstg)
{
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::OpenStorage(
		__RPC__in_opt_string const OLECHAR *pwcsName,
		__RPC__in_opt IStorage *pstgPriority,
		DWORD grfMode,
		__RPC__deref_opt_in_opt SNB snbExclude,
		DWORD reserved,
		__RPC__deref_out_opt IStorage **ppstg)
{
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::CopyTo(
		DWORD ciidExclude,
		const IID *rgiidExclude,
		__RPC__in_opt SNB snbExclude,
		IStorage *pstgDest)
{
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::MoveElementTo(
		__RPC__in_string const OLECHAR *pwcsName,
		__RPC__in_opt IStorage *pstgDest,
		__RPC__in_string const OLECHAR *pwcsNewName,
		DWORD grfFlags)
{
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::Commit(DWORD grfCommitFlags) {
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::Revert(void) {
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::EnumElements(DWORD reserved1,
		void *reserved2, DWORD reserved3, IEnumSTATSTG **ppenum) {
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::DestroyElement(
		__RPC__in_string const OLECHAR *pwcsName)
{
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::RenameElement(
		__RPC__in_string const OLECHAR *pwcsOldName,
		__RPC__in_string const OLECHAR *pwcsNewName)
{
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::SetElementTimes(
		__RPC__in_opt_string const OLECHAR *pwcsName,
		__RPC__in_opt const FILETIME *pctime,
		__RPC__in_opt const FILETIME *patime,
		__RPC__in_opt const FILETIME *pmtime)
{
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::SetClass(
		__RPC__in REFCLSID clsid)
{
	return S_OK;
}

HRESULT STDMETHODCALLTYPE WebView::SetStateBits(DWORD grfStateBits,
		DWORD grfMask) {
	return E_NOTIMPL;
}

HRESULT STDMETHODCALLTYPE WebView::Stat(
		__RPC__out STATSTG *pstatstg,
		DWORD grfStatFlag)
{
	return E_NOTIMPL;
}

static int webview_ie_compat_mode(DWORD ie_version) {
	HKEY hKey;
	TCHAR appname[MAX_PATH + 1];
	TCHAR *p;
	if (GetModuleFileName(NULL, appname, MAX_PATH + 1) == 0) {
		return -1;
	}
	for (p = &appname[strlen(appname) - 1]; p != appname && *p != '\\'; p--) {
	}
	p++;
	if (RegCreateKey(HKEY_CURRENT_USER, REGISTRY_BROWSER_EMULATION, &hKey)
			!= ERROR_SUCCESS) {
		return -1;
	}
	if (RegSetValueEx(hKey, p, 0, REG_DWORD, (BYTE *) &ie_version,
			sizeof(ie_version)) != ERROR_SUCCESS) {
		RegCloseKey(hKey);
		return -1;
	}
	RegCloseKey(hKey);
	return 0;
}

WebView * webview_new_with_parent(HWND parent) {
	return new WebView(parent);
}
void webview_delete(WebView * thisptr) {
	delete thisptr;
}
void webview_navigate(WebView * thisptr, wchar_t* szUrl) {
	thisptr->Navigate(szUrl);
}
HRESULT webview_url(WebView * thisptr, wchar_t** pszUrl) {
	return thisptr->LocationURL(pszUrl);
}
void webview_set_rect(WebView * thisptr, RECT rect) {
	thisptr->SetRect(rect);
}
