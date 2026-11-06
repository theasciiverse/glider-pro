#include "MainMenuUI.h"
#include "Environ.h"

#include "FontFamily.h"
#include "GpApplicationName.h"
#include "GliderProtos.h"
#include "Externs.h"
#include "IGpDisplayDriver.h"
#include "RenderedFont.h"
#include "GpRenderedFontMetrics.h"
#include "MainMenuUI.h"
#include "ResolveCachingColor.h"
#include "PLTimeTaggedVOSEvent.h"
#include "WindowDef.h"
#include "WindowManager.h"
#include "Vec2i.h"

#include "PLCore.h"
#include "PLDrivers.h"
#include "PLQDraw.h"
#include "PLStandardColors.h"
#include "PLSysCalls.h"

struct MainMenuControlState
{
	Window *m_window;

	int m_targetHorizontalCoordinate;
	Point m_dimensions;
	int m_page;
};

struct MainMenuUIState
{
	enum ControlID
	{
		Control_NewGame,
		Control_LoadSavedGame,
		Control_HighScores,
		Control_Page0To1,

		Control_LoadHouse,
		Control_Demo,
		Control_Page1To2,

		Control_AboutApplication,
		Control_AboutFramework,
		Control_Page2To0,

		Control_Count,
	};

	enum ControlStyle
	{
		ControlStyle_Icon,
		ControlStyle_Text,
	};

	MainMenuControlState m_controls[Control_Count];
	int m_scrollInOffset;
	int m_scrollInStep;
	int m_scrollOutDistance;
	int m_currentPage;

	ControlID m_activeControl;

	static const unsigned int kControlHeight = 40;
	static const unsigned int kControlLeftSpacing = 20;
	static const unsigned int kControlBottomSpacing = 20;
	static const unsigned int kControlIntermediateSpacing = 16;
	static const unsigned int kControlInteriorSpacing = 6;
	static const unsigned int kControlScrollInDecay = 32;
	static const unsigned int kControlScrollInDecayFalloffBits = 0;
	static const PortabilityLayer::FontPreset_t kControlFontPreset = PortabilityLayer::FontPresets::kHandwriting24;
};

static MainMenuUIState mainMenu;

static MainMenuUIState::ControlStyle GetControlStyleForControl(MainMenuUIState::ControlID controlID)
{
	switch (controlID)
	{
	case MainMenuUIState::Control_Page0To1:
	case MainMenuUIState::Control_Page1To2:
	case MainMenuUIState::Control_Page2To0:
		return MainMenuUIState::ControlStyle_Icon;

	default:
		return MainMenuUIState::ControlStyle_Text;
	};
}

static PLPasStr GetTextForControl(MainMenuUIState::ControlID controlID)
{
	switch (controlID)
	{
	case MainMenuUIState::Control_NewGame:
		return PSTR("New Game");
	case MainMenuUIState::Control_LoadSavedGame:
		return PSTR("Resume Game");
	case MainMenuUIState::Control_LoadHouse:
		return PSTR("Load House");
	case MainMenuUIState::Control_Demo:
		return PSTR("Demo");
	case MainMenuUIState::Control_HighScores:
		return PSTR("High Scores");
	case MainMenuUIState::Control_AboutApplication:
		return PSTR("About Glider PRO");
	case MainMenuUIState::Control_AboutFramework:
		return PSTR("About " GP_APPLICATION_NAME);

	default:
		return PSTR("");
	};
}

static void DrawMainMenuControl(DrawSurface *surface, MainMenuUIState::ControlID controlID, bool clicked)
{
	Rect surfaceRect = surface->m_port.GetRect();

	PortabilityLayer::ResolveCachingColor blackColor(StdColors::Black());
	PortabilityLayer::ResolveCachingColor whiteColor(StdColors::White());
	PortabilityLayer::ResolveCachingColor borderColor1(PortabilityLayer::RGBAColor::Create(255, 51, 51, 255));
	PortabilityLayer::ResolveCachingColor borderColor2(PortabilityLayer::RGBAColor::Create(255, 153, 51, 255));

	surface->FrameRect(surfaceRect, blackColor);
	surface->FrameRect(surfaceRect.Inset(1, 1), borderColor1);
	surface->FrameRect(surfaceRect.Inset(2, 2), borderColor2);

	if (clicked)
		surface->FillRect(surfaceRect.Inset(3, 3), borderColor2);
	else
		surface->FillRect(surfaceRect.Inset(3, 3), whiteColor);

	switch (GetControlStyleForControl(controlID))
	{
	case MainMenuUIState::ControlStyle_Text:
		{
			PortabilityLayer::RenderedFont *rfont = GetFont(MainMenuUIState::kControlFontPreset);
			if (!rfont)
				return;

			int32_t verticalOffset = (static_cast<int32_t>(surface->m_port.GetRect().Height()) + rfont->GetMetrics().m_ascent) / 2;

			surface->DrawString(Point::Create(MainMenuUIState::kControlInteriorSpacing, verticalOffset), GetTextForControl(controlID), blackColor, rfont);
		}
		break;
	case MainMenuUIState::ControlStyle_Icon:
		{
			const int dotSize = 4;
			const int dotSpacing = 2;

			const int dotCount = 3;
			const int dotTotalSize = dotSpacing * (dotCount - 1) + dotSize * dotCount;

			int32_t dotHorizontalOffset = (static_cast<int32_t>(surface->m_port.GetRect().Width()) - dotTotalSize) / 2;
			int32_t dotVerticalOffset = (static_cast<int32_t>(surface->m_port.GetRect().Height()) - dotSize) / 2;

			for (int i = 0; i < dotCount; i++)
			{
				int32_t hCoord = dotHorizontalOffset + i * (dotSize + dotSpacing);
				surface->FillEllipse(Rect::Create(dotVerticalOffset, hCoord, dotVerticalOffset + dotSize, hCoord + dotSize), blackColor);
			}
		}
		break;
	default:
		break;
	}
}

void StartScrollForPage()
{
	unsigned int displayHeight = PortabilityLayer::WindowManager::GetInstance()->GetDisplayResolution().m_y;

	DismissMainMenuUI();

	int page = mainMenu.m_currentPage;

	PortabilityLayer::WindowManager *wm = PortabilityLayer::WindowManager::GetInstance();

	int totalWidth = 0;

	for (int controlID = 0; controlID < MainMenuUIState::Control_Count; controlID++)
	{
		if (mainMenu.m_controls[controlID].m_page == page)
			totalWidth = mainMenu.m_controls[controlID].m_targetHorizontalCoordinate + mainMenu.m_controls[controlID].m_dimensions.h;
	}

	mainMenu.m_scrollInStep = 1;
	mainMenu.m_scrollInOffset = 0;
	while (mainMenu.m_scrollInOffset < totalWidth)
	{
		mainMenu.m_scrollInOffset += (mainMenu.m_scrollInStep >> MainMenuUIState::kControlScrollInDecayFalloffBits);
		mainMenu.m_scrollInStep += MainMenuUIState::kControlScrollInDecay;
	}

	mainMenu.m_scrollOutDistance = totalWidth;

	for (int i = 0; i < MainMenuUIState::Control_Count; i++)
	{
		MainMenuControlState &control = mainMenu.m_controls[i];

		if (control.m_page == page)
		{

			const uint16_t styleFlags = PortabilityLayer::WindowStyleFlags::kBorderless;

			PortabilityLayer::WindowDef wdef = PortabilityLayer::WindowDef::Create(Rect::Create(0, 0, control.m_dimensions.v, control.m_dimensions.h), styleFlags, true, 0, 0, PSTR(""));

			control.m_window = wm->CreateWindow(wdef);
			control.m_window->SetPosition(PortabilityLayer::Vec2i(control.m_targetHorizontalCoordinate - mainMenu.m_scrollInOffset, displayHeight - MainMenuUIState::kControlBottomSpacing - control.m_dimensions.v));
			wm->PutWindowBehind(control.m_window, wm->GetPutInFrontSentinel());

			DrawMainMenuControl(control.m_window->GetDrawSurface(), static_cast<MainMenuUIState::ControlID>(i), false);
		}
	}
}

static bool gs_useMainMenuUI = false;

void SetUseMainMenuUI(bool useMainMenuUI)
{
	gs_useMainMenuUI = useMainMenuUI;
}

bool UseMainMenuUI()
{
	return gs_useMainMenuUI || thisMac.isTouchscreen;
}

void StartMainMenuUI()
{
	DismissMainMenuUI();

	PortabilityLayer::RenderedFont *rfont = GetFont(MainMenuUIState::kControlFontPreset);
	if (!rfont)
		return;

	for (int controlID = 0; controlID < MainMenuUIState::Control_Count; controlID++)
	{
		MainMenuControlState &control = mainMenu.m_controls[controlID];

		MainMenuUIState::ControlStyle controlStyle = GetControlStyleForControl(static_cast<MainMenuUIState::ControlID>(controlID));

		if (controlStyle == MainMenuUIState::ControlStyle_Text)
		{
			size_t textLength = rfont->MeasurePStr(GetTextForControl(static_cast<MainMenuUIState::ControlID>(controlID)));

			control.m_dimensions.h = textLength + MainMenuUIState::kControlInteriorSpacing * 2;
			control.m_dimensions.v = MainMenuUIState::kControlHeight;
		}
		else
			control.m_dimensions.h = control.m_dimensions.v = MainMenuUIState::kControlHeight;
	}

	mainMenu.m_controls[MainMenuUIState::Control_LoadHouse].m_page = 1;
	mainMenu.m_controls[MainMenuUIState::Control_Demo].m_page = 1;
	mainMenu.m_controls[MainMenuUIState::Control_Page1To2].m_page = 1;

	mainMenu.m_controls[MainMenuUIState::Control_AboutApplication].m_page = 2;
	mainMenu.m_controls[MainMenuUIState::Control_AboutFramework].m_page = 2;
	mainMenu.m_controls[MainMenuUIState::Control_Page2To0].m_page = 2;

	for (int i = 0; i < MainMenuUIState::Control_Count; i++)
	{
		if (i == 0 || mainMenu.m_controls[i].m_page != mainMenu.m_controls[i - 1].m_page)
			mainMenu.m_controls[i].m_targetHorizontalCoordinate = MainMenuUIState::kControlLeftSpacing;
		else
			mainMenu.m_controls[i].m_targetHorizontalCoordinate = mainMenu.m_controls[i - 1].m_targetHorizontalCoordinate + mainMenu.m_controls[i - 1].m_dimensions.h + MainMenuUIState::kControlIntermediateSpacing;
	}

	mainMenu.m_currentPage = 0;
	StartScrollForPage();
}

static void DismissMainMenuUIPage()
{
	unsigned int displayHeight = PortabilityLayer::WindowManager::GetInstance()->GetDisplayResolution().m_y;

	PortabilityLayer::WindowManager *wm = PortabilityLayer::WindowManager::GetInstance();

	while (mainMenu.m_scrollInOffset < mainMenu.m_scrollOutDistance)
	{
		mainMenu.m_scrollInOffset += (mainMenu.m_scrollInStep >> MainMenuUIState::kControlScrollInDecayFalloffBits);
		mainMenu.m_scrollInStep += MainMenuUIState::kControlScrollInDecay;

		for (int i = 0; i < MainMenuUIState::Control_Count; i++)
		{
			MainMenuControlState &control = mainMenu.m_controls[i];
			if (control.m_page == mainMenu.m_currentPage)
				control.m_window->SetPosition(PortabilityLayer::Vec2i(control.m_targetHorizontalCoordinate - mainMenu.m_scrollInOffset, displayHeight - MainMenuUIState::kControlBottomSpacing - control.m_dimensions.v));
		}

		Delay(1, nullptr);
	}

	DismissMainMenuUI();
}


void TickMainMenuUI()
{
	if (mainMenu.m_scrollInOffset > 0)
	{
		PortabilityLayer::WindowManager *wm = PortabilityLayer::WindowManager::GetInstance();

		unsigned int displayHeight = PortabilityLayer::WindowManager::GetInstance()->GetDisplayResolution().m_y;

		mainMenu.m_scrollInStep -= MainMenuUIState::kControlScrollInDecay;
		mainMenu.m_scrollInOffset -= (mainMenu.m_scrollInStep >> MainMenuUIState::kControlScrollInDecayFalloffBits);

		for (int i = 0; i < MainMenuUIState::Control_Count; i++)
		{
			MainMenuControlState &control = mainMenu.m_controls[i];
			if (control.m_page == mainMenu.m_currentPage)
				control.m_window->SetPosition(PortabilityLayer::Vec2i(control.m_targetHorizontalCoordinate - mainMenu.m_scrollInOffset, displayHeight - MainMenuUIState::kControlBottomSpacing - control.m_dimensions.v));
		}
	}
}

void DismissMainMenuUI()
{
	PortabilityLayer::WindowManager *wm = PortabilityLayer::WindowManager::GetInstance();

	for (int i = 0; i < MainMenuUIState::Control_Count; i++)
	{
		MainMenuControlState &control = mainMenu.m_controls[i];

		if (control.m_window)
		{
			wm->DestroyWindow(control.m_window);
			control.m_window = nullptr;
		}
	}
}

void HandleMainMenuUIResolutionChange()
{
	PortabilityLayer::WindowManager *wm = PortabilityLayer::WindowManager::GetInstance();

	unsigned int displayHeight = PortabilityLayer::WindowManager::GetInstance()->GetDisplayResolution().m_y;

	for (int i = 0; i < MainMenuUIState::Control_Count; i++)
	{
		MainMenuControlState &control = mainMenu.m_controls[i];

		if (control.m_window)
		{
			PortabilityLayer::Vec2i pos = control.m_window->GetPosition();
			pos.m_y = displayHeight - MainMenuUIState::kControlBottomSpacing - MainMenuUIState::kControlHeight;
			pos.m_x = control.m_targetHorizontalCoordinate;
			control.m_window->SetPosition(pos);

			wm->PutWindowBehind(control.m_window, wm->GetPutInFrontSentinel());
		}
	}

	mainMenu.m_scrollInStep = 0;
	mainMenu.m_scrollInOffset = 0;
}

static void MainMenuUIMouseMove(Window *window, MainMenuUIState::ControlID controlID, const Point &localPoint)
{
	if (window->GetSurfaceRect().Contains(localPoint))
	{
		if (mainMenu.m_activeControl != controlID)
		{
			DrawMainMenuControl(window->GetDrawSurface(), controlID, true);
			mainMenu.m_activeControl = controlID;
		}
	}
	else
	{
		if (mainMenu.m_activeControl == controlID)
		{
			DrawMainMenuControl(window->GetDrawSurface(), controlID, false);
			mainMenu.m_activeControl = MainMenuUIState::Control_Count;
		}
	}
}

static void HandleMainMenuUISelection(MainMenuUIState::ControlID controlID)
{
	switch (controlID)
	{
	case MainMenuUIState::Control_NewGame:
		DismissMainMenuUIPage();
		DoGameMenu(iNewGame);
		StartMainMenuUI();
		break;

	case MainMenuUIState::Control_LoadSavedGame:
		DismissMainMenuUIPage();
		DoGameMenu(iOpenSavedGame);
		StartMainMenuUI();
		break;

	case MainMenuUIState::Control_HighScores:
		DismissMainMenuUIPage();
		DoOptionsMenu(iHighScores);
		StartMainMenuUI();
		break;

	case MainMenuUIState::Control_LoadHouse:
		DismissMainMenuUIPage();
		DoGameMenu(iLoadHouse);
		StartMainMenuUI();
		break;

	case MainMenuUIState::Control_Demo:
		DoDemoGame();	// This handles main menu UI by itself
		break;

	case MainMenuUIState::Control_AboutApplication:
		DismissMainMenuUIPage();
		DoAppleMenu(iAbout);
		StartScrollForPage();
		break;

	case MainMenuUIState::Control_AboutFramework:
		DismissMainMenuUIPage();
		DoAppleMenu(iAboutAerofoil);
		StartScrollForPage();
		break;

	case MainMenuUIState::Control_Page0To1:
		DismissMainMenuUIPage();
		mainMenu.m_currentPage = 1;
		StartScrollForPage();
		break;

	case MainMenuUIState::Control_Page1To2:
		DismissMainMenuUIPage();
		mainMenu.m_currentPage = 2;
		StartScrollForPage();
		break;

	case MainMenuUIState::Control_Page2To0:
		DismissMainMenuUIPage();
		mainMenu.m_currentPage = 0;
		StartScrollForPage();
		break;

	default:
		break;
	}
}

bool HandleMainMenuUIClick(Window *window, const Point &pt)
{
	MainMenuUIState::ControlID controlID = MainMenuUIState::Control_Count;

	if (mainMenu.m_scrollInOffset != 0)
		return true;

	for (int i = 0; i < MainMenuUIState::Control_Count; i++)
	{
		const MainMenuControlState &control = mainMenu.m_controls[i];

		if (control.m_window == window)
		{
			controlID = static_cast<MainMenuUIState::ControlID>(i);
			break;
		}
	}

	if (controlID == MainMenuUIState::Control_Count)
		return false;

	DrawMainMenuControl(mainMenu.m_controls[controlID].m_window->GetDrawSurface(), controlID, true);

	mainMenu.m_activeControl = controlID;

	for (;;)
	{
		TimeTaggedVOSEvent evt;
		if (WaitForEvent(&evt, 1))
		{
			if (evt.m_vosEvent.m_eventType == GpVOSEventTypes::kMouseInput)
			{
				const GpMouseInputEvent &mouseEvt = evt.m_vosEvent.m_event.m_mouseInputEvent;

				if (mouseEvt.m_eventType == GpMouseEventTypes::kLeave)
					return true;

				if (mouseEvt.m_eventType == GpMouseEventTypes::kMove || mouseEvt.m_eventType == GpMouseEventTypes::kDown || mouseEvt.m_eventType == GpMouseEventTypes::kUp)
				{
					MainMenuUIMouseMove(window, controlID, window->MouseToLocal(mouseEvt));
				}

				if (mouseEvt.m_eventType == GpMouseEventTypes::kUp)
				{
					if (mainMenu.m_activeControl != MainMenuUIState::Control_Count)
					{
						DrawMainMenuControl(mainMenu.m_controls[controlID].m_window->GetDrawSurface(), controlID, false);
						HandleMainMenuUISelection(controlID);
					}
					return true;
				}
			}
		}
	}

	return true;
}
