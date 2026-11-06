// Terminal display driver for Aerofoil, on notcurses.
//
// The game draws into its own surfaces at a fixed 640x480 virtual screen.
// This driver keeps a copy of every surface, composites the surfaces the
// game draws each frame into one RGBA frame in software (the same work the
// SDL driver's shaders do: palette lookup, darken, the flicker wipe,
// desaturation), and hands the frame to notcurses, which shows it with the
// best blitter the terminal has (pixels where it can, else sextants,
// quadrants or half blocks).
//
// It also owns keyboard and mouse, read through notcurses. Terminals with
// the kitty keyboard protocol report presses and releases exactly; others
// only send presses and repeats, so there a key counts as held while its
// repeats keep arriving (as in games/glider). Gamepads and sound stay with
// Aerofoil's SDL drivers, which need no window: SDL events are pumped here
// and gamepad events passed on.

#include "IGpDisplayDriver.h"
#include "IGpDisplayDriverSurface.h"
#include "IGpCursor.h"
#include "IGpLogDriver.h"
#include "IGpVOSEventQueue.h"
#include "GpDisplayDriverProperties.h"
#include "GpDisplayDriverTickStatus.h"
#include "GpVOSEvent.h"
#include "GpInputDriver_SDL_Gamepad.h"

#include "SDL.h"

#include <notcurses/notcurses.h>

#include <algorithm>
#include <chrono>
#include <new>
#include <stdlib.h>
#include <string.h>
#include <thread>
#include <vector>

namespace
{
	const unsigned int kVirtualWidth = 640;
	const unsigned int kVirtualHeight = 480;

	// Without key releases: a press counts as held this long, until the
	// terminal's repeats start, then as long as they keep coming.
	const std::chrono::milliseconds kPreRepeat(550);
	const std::chrono::milliseconds kRepeatGap(100);

	typedef std::chrono::steady_clock Clock;

	struct KeyId
	{
		GpKeyIDSubset_t m_subset;
		GpKeyboardInputEvent::KeyUnion m_key;

		bool operator==(const KeyId &other) const
		{
			if (m_subset != other.m_subset)
				return false;
			switch (m_subset)
			{
			case GpKeyIDSubsets::kASCII:
				return m_key.m_asciiChar == other.m_key.m_asciiChar;
			case GpKeyIDSubsets::kUnicode:
				return m_key.m_unicodeChar == other.m_key.m_unicodeChar;
			case GpKeyIDSubsets::kSpecial:
				return m_key.m_specialKey == other.m_key.m_specialKey;
			case GpKeyIDSubsets::kFKey:
				return m_key.m_fKey == other.m_key.m_fKey;
			case GpKeyIDSubsets::kNumPadNumber:
				return m_key.m_numPadNumber == other.m_key.m_numPadNumber;
			case GpKeyIDSubsets::kNumPadSpecial:
				return m_key.m_numPadSpecialKey == other.m_key.m_numPadSpecialKey;
			default:
				return false;
			}
		}
	};

	struct HeldKey
	{
		KeyId m_id;
		Clock::time_point m_last;
		unsigned int m_repeats;
	};
}

class GpDisplayDriver_Notcurses;

class GpDisplayDriverSurface_Notcurses final : public IGpDisplayDriverSurface
{
public:
	GpDisplayDriverSurface_Notcurses(size_t width, size_t height, GpPixelFormat_t pixelFormat)
		: m_width(width)
		, m_height(height)
		, m_pixelFormat(pixelFormat)
		, m_bytesPerPixel(BytesPerPixel(pixelFormat))
		, m_pixels(width * height * BytesPerPixel(pixelFormat), 0)
	{
	}

	void Upload(const void *data, size_t x, size_t y, size_t width, size_t height, size_t pitch) override
	{
		// Rows in the same order as UploadEntire, as the SDL driver's TexSubImage2D.
		const uint8_t *src = static_cast<const uint8_t*>(data);
		for (size_t row = 0; row < height; row++)
			memcpy(&m_pixels[((y + row) * m_width + x) * m_bytesPerPixel], src + row * pitch, width * m_bytesPerPixel);
	}

	void UploadEntire(const void *data, size_t pitch) override
	{
		const uint8_t *src = static_cast<const uint8_t*>(data);
		const size_t rowBytes = m_width * m_bytesPerPixel;
		for (size_t row = 0; row < m_height; row++)
			memcpy(&m_pixels[row * rowBytes], src + row * pitch, rowBytes);
	}

	void Destroy() override
	{
		delete this;
	}

	static size_t BytesPerPixel(GpPixelFormat_t format)
	{
		switch (format)
		{
		case GpPixelFormats::kRGB32:
			return 4;
		case GpPixelFormats::kRGB24:
			return 3;
		case GpPixelFormats::kRGB555:
			return 2;
		default:
			return 1;
		}
	}

	size_t m_width;
	size_t m_height;
	GpPixelFormat_t m_pixelFormat;
	size_t m_bytesPerPixel;
	std::vector<uint8_t> m_pixels;
};

class GpCursor_Notcurses final : public IGpCursor
{
public:
	void Destroy() override
	{
		delete this;
	}
};

class GpDisplayDriver_Notcurses final : public IGpDisplayDriver
{
public:
	explicit GpDisplayDriver_Notcurses(const GpDisplayDriverProperties &properties);

	bool Init() override;
	void ServeTicks(int tickCount) override;
	void ForceSync() override;
	void Shutdown() override;

	void GetInitialDisplayResolution(unsigned int *width, unsigned int *height) override;
	IGpDisplayDriverSurface *CreateSurface(size_t width, size_t height, size_t pitch, GpPixelFormat_t pixelFormat, SurfaceInvalidateCallback_t invalidateCallback, void *invalidateContext) override;
	void DrawSurface(IGpDisplayDriverSurface *surface, int32_t x, int32_t y, size_t width, size_t height, const GpDisplayDriverSurfaceEffects *effects) override;
	IGpCursor *CreateBWCursor(size_t width, size_t height, const void *pixelData, const void *maskData, size_t hotSpotX, size_t hotSpotY) override;
	IGpCursor *CreateColorCursor(size_t width, size_t height, const void *pixelDataRGBA, size_t hotSpotX, size_t hotSpotY) override;
	void SetCursor(IGpCursor *cursor) override;
	void SetStandardCursor(EGpStandardCursor_t standardCursor) override;
	void UpdatePalette(const void *paletteData) override;
	void SetBackgroundColor(uint8_t r, uint8_t g, uint8_t b, uint8_t a) override;
	void SetBackgroundDarkenEffect(bool isDark) override;
	void SetUseICCProfile(bool useICCProfile) override;
	void RequestToggleFullScreen(uint32_t timestamp) override;
	void RequestResetVirtualResolution() override;
	bool IsFullScreen() const override;
	const GpDisplayDriverProperties &GetProperties() const override;
	IGpPrefsHandler *GetPrefsHandler() const override;

private:
	void RenderFrame();
	void Present();
	void PollInput();
	void PumpSDL();
	void ReleaseStaleKeys(Clock::time_point now);
	void TranslateKey(const ncinput &ni);
	void TranslateMouse(const ncinput &ni);
	bool IdentifyKey(const ncinput &ni, KeyId &outId) const;
	void PostKey(GpKeyboardInputEventType_t type, const KeyId &id, uint32_t repeatCount);
	void PostChar(GpKeyboardInputEventType_t type, uint32_t codePoint);
	void PostMouse(GpMouseEventType_t type, GpMouseButton_t button, int cellY, int cellX, int pixelY, int pixelX);
	void Log(IGpLogDriver::Category category, const char *fmt, const char *arg) const;

	GpDisplayDriverProperties m_properties;

	struct notcurses *m_nc;
	struct ncplane *m_plane;
	ncblitter_e m_blitter;
	bool m_haveReleases;

	std::vector<uint32_t> m_frame;	// RGBA bytes, kVirtualWidth x kVirtualHeight
	uint8_t m_palette[256 * 4];
	uint8_t m_bgColor[4];
	bool m_bgIsDark;

	std::vector<HeldKey> m_held;
	GpMouseButton_t m_mouseHeld;

	Clock::time_point m_nextFrame;
	bool m_quitRequested;
};

GpDisplayDriver_Notcurses::GpDisplayDriver_Notcurses(const GpDisplayDriverProperties &properties)
	: m_properties(properties)
	, m_nc(nullptr)
	, m_plane(nullptr)
	, m_blitter(NCBLIT_DEFAULT)
	, m_haveReleases(false)
	, m_frame(kVirtualWidth * kVirtualHeight, 0)
	, m_bgIsDark(false)
	, m_mouseHeld(GpMouseButtons::kNone)
	, m_quitRequested(false)
{
	memset(m_palette, 0, sizeof(m_palette));
	m_bgColor[0] = m_bgColor[1] = m_bgColor[2] = 0;
	m_bgColor[3] = 255;
}

void GpDisplayDriver_Notcurses::Log(IGpLogDriver::Category category, const char *fmt, const char *arg) const
{
	if (m_properties.m_logger)
		m_properties.m_logger->Printf(category, fmt, arg);
}

bool GpDisplayDriver_Notcurses::Init()
{
	notcurses_options opts;
	memset(&opts, 0, sizeof(opts));
	opts.flags = NCOPTION_SUPPRESS_BANNERS;
	opts.loglevel = NCLOGLEVEL_SILENT;

	m_nc = notcurses_core_init(&opts, nullptr);
	if (!m_nc)
	{
		Log(IGpLogDriver::Category_Error, "%s", "notcurses_core_init failed");
		return false;
	}

	notcurses_mice_enable(m_nc, NCMICE_ALL_EVENTS);
	notcurses_cursor_disable(m_nc);

	// Pixels where the terminal has them, else the finest cell blitter.
	// tmux and screen claim pixel graphics whatever the terminal around them can do.
	const bool multiplexed = getenv("TMUX") || getenv("STY");
	if (notcurses_canpixel(m_nc) && !multiplexed)
		m_blitter = NCBLIT_PIXEL;
	else if (notcurses_cansextant(m_nc))
		m_blitter = NCBLIT_3x2;
	else if (notcurses_canquadrant(m_nc))
		m_blitter = NCBLIT_2x2;
	else
		m_blitter = NCBLIT_2x1;

	const char *blitterName = notcurses_str_blitter(m_blitter);
	Log(IGpLogDriver::Category_Information, "notcurses started, blitter %s", blitterName ? blitterName : "?");

	m_nextFrame = Clock::now();
	return true;
}

void GpDisplayDriver_Notcurses::ServeTicks(int tickCount)
{
	const Clock::duration frameTime = std::chrono::duration_cast<Clock::duration>(std::chrono::duration<double>(
		static_cast<double>(m_properties.m_frameTimeLockNumerator) / static_cast<double>(m_properties.m_frameTimeLockDenominator)));

	while (tickCount > 0 && !m_quitRequested)
	{
		PollInput();
		PumpSDL();

		Clock::time_point now = Clock::now();
		ReleaseStaleKeys(now);

		if (now < m_nextFrame)
		{
			std::this_thread::sleep_for(std::min<Clock::duration>(m_nextFrame - now, std::chrono::milliseconds(2)));
			continue;
		}

		// Fell far behind (a suspended terminal, a slow blit): don't race to catch up.
		if (now - m_nextFrame > frameTime * 4)
			m_nextFrame = now;
		m_nextFrame += frameTime;

		RenderFrame();
		Present();
		tickCount--;
	}
}

void GpDisplayDriver_Notcurses::ForceSync()
{
	m_nextFrame = Clock::now();
}

void GpDisplayDriver_Notcurses::Shutdown()
{
	if (m_plane)
	{
		ncplane_destroy(m_plane);
		m_plane = nullptr;
	}
	if (m_nc)
	{
		notcurses_stop(m_nc);
		m_nc = nullptr;
	}
	this->~GpDisplayDriver_Notcurses();
	free(this);
}

void GpDisplayDriver_Notcurses::GetInitialDisplayResolution(unsigned int *width, unsigned int *height)
{
	*width = kVirtualWidth;
	*height = kVirtualHeight;
}

IGpDisplayDriverSurface *GpDisplayDriver_Notcurses::CreateSurface(size_t width, size_t height, size_t pitch, GpPixelFormat_t pixelFormat, SurfaceInvalidateCallback_t invalidateCallback, void *invalidateContext)
{
	(void)pitch;
	(void)invalidateCallback;
	(void)invalidateContext;
	return new (std::nothrow) GpDisplayDriverSurface_Notcurses(width, height, pixelFormat);
}

void GpDisplayDriver_Notcurses::DrawSurface(IGpDisplayDriverSurface *surface, int32_t x, int32_t y, size_t width, size_t height, const GpDisplayDriverSurfaceEffects *effects)
{
	static const GpDisplayDriverSurfaceEffects defaultEffects;
	if (!effects)
		effects = &defaultEffects;

	const GpDisplayDriverSurface_Notcurses *s = static_cast<const GpDisplayDriverSurface_Notcurses*>(surface);
	const GpPixelFormat_t format = s->m_pixelFormat;
	const bool paletted = (format == GpPixelFormats::k8BitStandard || format == GpPixelFormats::k8BitCustom);
	if (!paletted && format != GpPixelFormats::kRGB32)
		return;	// As the SDL driver: nothing else is drawn

	const int modulation = effects->m_darken ? 128 : 256;
	const float desaturation = effects->m_desaturation;

	for (size_t row = 0; row < height; row++)
	{
		const int32_t dy = y + static_cast<int32_t>(row);
		if (dy < 0 || dy >= static_cast<int32_t>(kVirtualHeight))
			continue;

		// The quad stretches the surface over width x height, nearest pixel.
		const size_t sy = std::min(s->m_height - 1, row * s->m_height / height);

		for (size_t col = 0; col < width; col++)
		{
			const int32_t dx = x + static_cast<int32_t>(col);
			if (dx < 0 || dx >= static_cast<int32_t>(kVirtualWidth))
				continue;

			const size_t sx = std::min(s->m_width - 1, col * s->m_width / width);
			const uint8_t *p = &s->m_pixels[(sy * s->m_width + sx) * s->m_bytesPerPixel];

			int r, g, b;
			if (paletted)
			{
				const uint8_t *c = &m_palette[p[0] * 4];
				r = c[0];
				g = c[1];
				b = c[2];
			}
			else
			{
				r = p[0];
				g = p[1];
				b = p[2];
			}

			r = r * modulation / 256;
			g = g * modulation / 256;
			b = b * modulation / 256;

			if (effects->m_flicker)
			{
				// The window wipe: dark before the line, white on it, the image after.
				const int32_t t = effects->m_flickerAxisX * static_cast<int32_t>(sx) + effects->m_flickerAxisY * static_cast<int32_t>(sy);
				if (t < effects->m_flickerStartThreshold)
					continue;
				if (t < effects->m_flickerEndThreshold)
					r = g = b = 255;
			}

			if (desaturation != 0.0f && !(r == 255 && g == 255 && b == 0))
			{
				const float gray = (r * 3.0f + g * 6.0f + b * 1.0f) / 10.0f;
				r = static_cast<int>(r * (1.0f - desaturation) + gray * desaturation);
				g = static_cast<int>(g * (1.0f - desaturation) + gray * desaturation);
				b = static_cast<int>(b * (1.0f - desaturation) + gray * desaturation);
			}

			uint8_t *out = reinterpret_cast<uint8_t*>(&m_frame[dy * kVirtualWidth + dx]);
			out[0] = static_cast<uint8_t>(r);
			out[1] = static_cast<uint8_t>(g);
			out[2] = static_cast<uint8_t>(b);
			out[3] = 255;
		}
	}
}

IGpCursor *GpDisplayDriver_Notcurses::CreateBWCursor(size_t, size_t, const void *, const void *, size_t, size_t)
{
	// The terminal's own pointer stays visible over the game.
	return new (std::nothrow) GpCursor_Notcurses();
}

IGpCursor *GpDisplayDriver_Notcurses::CreateColorCursor(size_t, size_t, const void *, size_t, size_t)
{
	return new (std::nothrow) GpCursor_Notcurses();
}

void GpDisplayDriver_Notcurses::SetCursor(IGpCursor *)
{
}

void GpDisplayDriver_Notcurses::SetStandardCursor(EGpStandardCursor_t)
{
}

void GpDisplayDriver_Notcurses::UpdatePalette(const void *paletteData)
{
	memcpy(m_palette, paletteData, sizeof(m_palette));
}

void GpDisplayDriver_Notcurses::SetBackgroundColor(uint8_t r, uint8_t g, uint8_t b, uint8_t a)
{
	m_bgColor[0] = r;
	m_bgColor[1] = g;
	m_bgColor[2] = b;
	m_bgColor[3] = a;
}

void GpDisplayDriver_Notcurses::SetBackgroundDarkenEffect(bool isDark)
{
	m_bgIsDark = isDark;
}

void GpDisplayDriver_Notcurses::SetUseICCProfile(bool)
{
}

void GpDisplayDriver_Notcurses::RequestToggleFullScreen(uint32_t)
{
}

void GpDisplayDriver_Notcurses::RequestResetVirtualResolution()
{
}

bool GpDisplayDriver_Notcurses::IsFullScreen() const
{
	return false;
}

const GpDisplayDriverProperties &GpDisplayDriver_Notcurses::GetProperties() const
{
	return m_properties;
}

IGpPrefsHandler *GpDisplayDriver_Notcurses::GetPrefsHandler() const
{
	return nullptr;
}

void GpDisplayDriver_Notcurses::RenderFrame()
{
	uint8_t bg[4];
	for (int i = 0; i < 3; i++)
		bg[i] = m_bgIsDark ? m_bgColor[i] / 4 : m_bgColor[i];
	bg[3] = 255;

	uint32_t bgPixel;
	memcpy(&bgPixel, bg, 4);
	std::fill(m_frame.begin(), m_frame.end(), bgPixel);

	m_properties.m_renderFunc(m_properties.m_renderFuncContext);
}

void GpDisplayDriver_Notcurses::Present()
{
	struct ncvisual *ncv = ncvisual_from_rgba(m_frame.data(), kVirtualHeight, kVirtualWidth * 4, kVirtualWidth);
	if (!ncv)
		return;

	// The 640x480 screen, scaled to fit the terminal with its aspect kept,
	// centred; a fresh plane each frame so a resized terminal just works.
	struct ncvisual_options vopts;
	memset(&vopts, 0, sizeof(vopts));
	vopts.n = m_plane;
	vopts.scaling = NCSCALE_SCALE;
	vopts.blitter = m_blitter;
	vopts.y = NCALIGN_CENTER;
	vopts.x = NCALIGN_CENTER;
	vopts.flags = NCVISUAL_OPTION_HORALIGNED | NCVISUAL_OPTION_VERALIGNED | NCVISUAL_OPTION_CHILDPLANE;

	if (!m_plane)
	{
		vopts.n = notcurses_stdplane(m_nc);
		m_plane = ncvisual_blit(m_nc, ncv, &vopts);
	}
	else
	{
		ncplane_destroy(m_plane);
		vopts.n = notcurses_stdplane(m_nc);
		m_plane = ncvisual_blit(m_nc, ncv, &vopts);
	}

	ncvisual_destroy(ncv);
	notcurses_render(m_nc);
}

void GpDisplayDriver_Notcurses::PumpSDL()
{
	SDL_Event evt;
	while (SDL_PollEvent(&evt))
	{
		switch (evt.type)
		{
		case SDL_CONTROLLERAXISMOTION:
		case SDL_CONTROLLERBUTTONDOWN:
		case SDL_CONTROLLERBUTTONUP:
		case SDL_CONTROLLERDEVICEADDED:
		case SDL_CONTROLLERDEVICEREMOVED:
		case SDL_CONTROLLERDEVICEREMAPPED:
			if (IGpInputDriverSDLGamepad *gamepadDriver = IGpInputDriverSDLGamepad::GetInstance())
				gamepadDriver->ProcessSDLEvent(evt);
			break;
		case SDL_QUIT:
			if (GpVOSEvent *qevt = m_properties.m_eventQueue->QueueEvent())
				qevt->m_eventType = GpVOSEventTypes::kQuit;
			break;
		default:
			break;
		}
	}
}

void GpDisplayDriver_Notcurses::PollInput()
{
	ncinput ni;
	for (;;)
	{
		memset(&ni, 0, sizeof(ni));
		const uint32_t id = notcurses_get_nblock(m_nc, &ni);
		if (id == 0 || id == static_cast<uint32_t>(-1))
			break;

		if (id == NCKEY_RESIZE)
			continue;	// Present() fits the next frame to the new size

		if (nckey_mouse_p(id))
			TranslateMouse(ni);
		else
			TranslateKey(ni);
	}
}

bool GpDisplayDriver_Notcurses::IdentifyKey(const ncinput &ni, KeyId &outId) const
{
	const uint32_t id = ni.id;
	GpKeyboardInputEvent::KeyUnion key;
	memset(&key, 0, sizeof(key));

	GpKeyIDSubset_t subset = GpKeyIDSubsets::kSpecial;
	switch (id)
	{
	case NCKEY_UP: key.m_specialKey = GpKeySpecials::kUpArrow; break;
	case NCKEY_DOWN: key.m_specialKey = GpKeySpecials::kDownArrow; break;
	case NCKEY_LEFT: key.m_specialKey = GpKeySpecials::kLeftArrow; break;
	case NCKEY_RIGHT: key.m_specialKey = GpKeySpecials::kRightArrow; break;
	case NCKEY_ENTER: case '\r': case '\n': key.m_specialKey = GpKeySpecials::kEnter; break;
	case NCKEY_ESC: key.m_specialKey = GpKeySpecials::kEscape; break;
	case NCKEY_TAB: key.m_specialKey = GpKeySpecials::kTab; break;
	case NCKEY_BACKSPACE: case 0x7f: case 0x08: key.m_specialKey = GpKeySpecials::kBackspace; break;
	case NCKEY_DEL: key.m_specialKey = GpKeySpecials::kDelete; break;
	case NCKEY_INS: key.m_specialKey = GpKeySpecials::kInsert; break;
	case NCKEY_HOME: key.m_specialKey = GpKeySpecials::kHome; break;
	case NCKEY_END: key.m_specialKey = GpKeySpecials::kEnd; break;
	case NCKEY_PGUP: key.m_specialKey = GpKeySpecials::kPageUp; break;
	case NCKEY_PGDOWN: key.m_specialKey = GpKeySpecials::kPageDown; break;
	case NCKEY_PAUSE: key.m_specialKey = GpKeySpecials::kPause; break;
	case NCKEY_CAPS_LOCK: key.m_specialKey = GpKeySpecials::kCapsLock; break;
	case NCKEY_LSHIFT: key.m_specialKey = GpKeySpecials::kLeftShift; break;
	case NCKEY_RSHIFT: key.m_specialKey = GpKeySpecials::kRightShift; break;
	case NCKEY_LCTRL: key.m_specialKey = GpKeySpecials::kLeftCtrl; break;
	case NCKEY_RCTRL: key.m_specialKey = GpKeySpecials::kRightCtrl; break;
	case NCKEY_LALT: key.m_specialKey = GpKeySpecials::kLeftAlt; break;
	case NCKEY_RALT: key.m_specialKey = GpKeySpecials::kRightAlt; break;
	default:
		if (id >= NCKEY_F01 && id <= NCKEY_F12)
		{
			subset = GpKeyIDSubsets::kFKey;
			key.m_fKey = static_cast<unsigned char>(id - NCKEY_F00);
		}
		else if (nckey_synthesized_p(id))
			return false;
		else if (id < 128)
		{
			// Letters are reported as the key, lower case, as SDL's keysyms are.
			subset = GpKeyIDSubsets::kASCII;
			char c = static_cast<char>(id);
			if (c >= 'A' && c <= 'Z')
				c = static_cast<char>(c - 'A' + 'a');
			key.m_asciiChar = c;
		}
		else
		{
			subset = GpKeyIDSubsets::kUnicode;
			key.m_unicodeChar = id;
		}
		break;
	}

	outId.m_subset = subset;
	outId.m_key = key;
	return true;
}

void GpDisplayDriver_Notcurses::PostKey(GpKeyboardInputEventType_t type, const KeyId &id, uint32_t repeatCount)
{
	if (GpVOSEvent *evt = m_properties.m_eventQueue->QueueEvent())
	{
		evt->m_eventType = GpVOSEventTypes::kKeyboardInput;
		GpKeyboardInputEvent &k = evt->m_event.m_keyboardInputEvent;
		k.m_eventType = type;
		k.m_keyIDSubset = id.m_subset;
		k.m_key = id.m_key;
		k.m_repeatCount = repeatCount;
	}
}

void GpDisplayDriver_Notcurses::PostChar(GpKeyboardInputEventType_t type, uint32_t codePoint)
{
	KeyId id;
	memset(&id, 0, sizeof(id));
	if (codePoint < 128)
	{
		id.m_subset = GpKeyIDSubsets::kASCII;
		id.m_key.m_asciiChar = static_cast<char>(codePoint);
	}
	else
	{
		id.m_subset = GpKeyIDSubsets::kUnicode;
		id.m_key.m_unicodeChar = codePoint;
	}
	PostKey(type, id, 1);
}

void GpDisplayDriver_Notcurses::TranslateKey(const ncinput &ni)
{
	if (ni.evtype == NCTYPE_RELEASE)
		m_haveReleases = true;

	// Ctrl-C quits, as in every terminal game.
	if (ni.id == 'c' && ncinput_ctrl_p(&ni) && ni.evtype != NCTYPE_RELEASE)
	{
		if (GpVOSEvent *evt = m_properties.m_eventQueue->QueueEvent())
			evt->m_eventType = GpVOSEventTypes::kQuit;
		m_quitRequested = true;
		return;
	}

	KeyId id;
	if (!IdentifyKey(ni, id))
		return;

	const Clock::time_point now = Clock::now();
	std::vector<HeldKey>::iterator held = std::find_if(m_held.begin(), m_held.end(), [&](const HeldKey &h) { return h.m_id == id; });

	if (ni.evtype == NCTYPE_RELEASE)
	{
		if (held != m_held.end())
		{
			PostKey(GpKeyboardInputEventTypes::kUp, id, 1);
			m_held.erase(held);
		}
		return;
	}

	const bool isRepeat = (ni.evtype == NCTYPE_REPEAT) || (held != m_held.end());
	PostKey(isRepeat ? GpKeyboardInputEventTypes::kAuto : GpKeyboardInputEventTypes::kDown, id, 1);

	// Typed text: printable characters, and Enter as '\n' as the SDL driver sends it.
	const GpKeyboardInputEventType_t charType = isRepeat ? GpKeyboardInputEventTypes::kAutoChar : GpKeyboardInputEventTypes::kDownChar;
	if (id.m_subset == GpKeyIDSubsets::kSpecial && id.m_key.m_specialKey == GpKeySpecials::kEnter)
		PostChar(charType, '\n');
	else if (!nckey_synthesized_p(ni.id) && ni.id >= 32 && ni.id != 0x7f && !ncinput_ctrl_p(&ni))
	{
		uint32_t cp = ni.id;
		if (ni.eff_text[0] != 0)
			cp = ni.eff_text[0];	// Shifted and composed characters
		PostChar(charType, cp);
	}

	if (held != m_held.end())
	{
		held->m_last = now;
		held->m_repeats++;
	}
	else
	{
		HeldKey h;
		h.m_id = id;
		h.m_last = now;
		h.m_repeats = 0;
		m_held.push_back(h);
	}
}

void GpDisplayDriver_Notcurses::ReleaseStaleKeys(Clock::time_point now)
{
	if (m_haveReleases)
		return;

	for (size_t i = 0; i < m_held.size();)
	{
		const HeldKey &h = m_held[i];
		const Clock::duration window = (h.m_repeats == 0) ? Clock::duration(kPreRepeat) : Clock::duration(kRepeatGap);
		if (now - h.m_last > window)
		{
			PostKey(GpKeyboardInputEventTypes::kUp, h.m_id, 1);
			m_held.erase(m_held.begin() + i);
		}
		else
			i++;
	}
}

void GpDisplayDriver_Notcurses::PostMouse(GpMouseEventType_t type, GpMouseButton_t button, int cellY, int cellX, int pixelY, int pixelX)
{
	if (!m_plane)
		return;

	// Where the game's screen sits on the terminal, and how big it is in pixels.
	int planeY = 0, planeX = 0;
	ncplane_abs_yx(m_plane, &planeY, &planeX);
	unsigned int rows = 0, cols = 0;
	ncplane_dim_yx(m_plane, &rows, &cols);
	if (rows == 0 || cols == 0)
		return;

	unsigned int cellPixY = 0, cellPixX = 0;
	ncplane_pixel_geom(m_plane, nullptr, nullptr, &cellPixY, &cellPixX, nullptr, nullptr);

	double fy, fx;
	if (cellPixY > 0 && cellPixX > 0 && pixelY >= 0 && pixelX >= 0)
	{
		fy = ((cellY - planeY) * static_cast<double>(cellPixY) + pixelY) / (rows * static_cast<double>(cellPixY));
		fx = ((cellX - planeX) * static_cast<double>(cellPixX) + pixelX) / (cols * static_cast<double>(cellPixX));
	}
	else
	{
		fy = (cellY - planeY + 0.5) / rows;
		fx = (cellX - planeX + 0.5) / cols;
	}

	if (GpVOSEvent *evt = m_properties.m_eventQueue->QueueEvent())
	{
		evt->m_eventType = GpVOSEventTypes::kMouseInput;
		GpMouseInputEvent &m = evt->m_event.m_mouseInputEvent;
		m.m_x = static_cast<int32_t>(std::max(0.0, std::min(fx, 0.9999)) * kVirtualWidth);
		m.m_y = static_cast<int32_t>(std::max(0.0, std::min(fy, 0.9999)) * kVirtualHeight);
		m.m_eventType = type;
		m.m_button = button;
	}
}

void GpDisplayDriver_Notcurses::TranslateMouse(const ncinput &ni)
{
	GpMouseButton_t button = GpMouseButtons::kNone;
	switch (ni.id)
	{
	case NCKEY_BUTTON1: button = GpMouseButtons::kLeft; break;
	case NCKEY_BUTTON2: button = GpMouseButtons::kMiddle; break;
	case NCKEY_BUTTON3: button = GpMouseButtons::kRight; break;
	case NCKEY_MOTION: button = GpMouseButtons::kNone; break;
	default: return;	// Wheel and extra buttons: not used by the game
	}

	GpMouseEventType_t type = GpMouseEventTypes::kMove;
	if (button != GpMouseButtons::kNone)
	{
		if (ni.evtype == NCTYPE_RELEASE)
		{
			type = GpMouseEventTypes::kUp;
			m_mouseHeld = GpMouseButtons::kNone;
		}
		else if (m_mouseHeld == button)
			type = GpMouseEventTypes::kMove;	// Drag
		else
		{
			type = GpMouseEventTypes::kDown;
			m_mouseHeld = button;
		}
	}

	PostMouse(type, type == GpMouseEventTypes::kMove ? GpMouseButtons::kNone : button, ni.y, ni.x, ni.ypx, ni.xpx);
}

IGpDisplayDriver *GpDriver_CreateDisplayDriver_Notcurses(const GpDisplayDriverProperties &properties)
{
	void *mem = malloc(sizeof(GpDisplayDriver_Notcurses));
	if (!mem)
		return nullptr;
	return new (mem) GpDisplayDriver_Notcurses(properties);
}
