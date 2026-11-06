
//============================================================================
//----------------------------------------------------------------------------
//									 Play.c
//----------------------------------------------------------------------------
//============================================================================


#include "PLResources.h"
#include "PLStandardColors.h"
#include "DisplayDeviceManager.h"
#include "Externs.h"
#include "Environ.h"
#include "House.h"
#include "MainMenuUI.h"
#include "MainWindow.h"
#include "PLEventQueue.h"
#include "PLTimeTaggedVOSEvent.h"
#include "RectUtils.h"
#include "ResolveCachingColor.h"
#include "Scoreboard.h"
#include "Utilities.h"


#define kHouseBannerAlert		1009
#define kInitialGliders			2
#define kRingDelay				90
#define kRingSpread				25000	// 25000
#define kRingBaseDelay			5000	// 5000
#define kChimeDelay				180


typedef struct
{
	short		nextRing;
	short		rings;
	short		delay;
} phoneType, *phonePtr;


void InitGlider (gliderPtr, short);
void SetHouseToFirstRoom (void);
void SetHouseToSavedRoom (void);
void PlayGame (void);
void HandleRoomVisitation (void);
void SetObjectsToDefaults (void);
void InitTelephone (void);
void HandleTelephone (void);
void DoEndGame (void);

phoneType	thePhone, theChimes;
Rect		glidSrcRect, justRoomsRect;
DrawSurface	*glidSrcMap, *glid2SrcMap;
DrawSurface	*glidMaskMap;
long		gameFrame;
short		batteryTotal, bandsTotal, foilTotal, mortals;
Boolean		playing, evenFrame, twoPlayerGame, showFoil, demoGoing;
Boolean		doBackground, playerSuicide, phoneBitSet, tvOn;
Boolean		pendingTouchScreenMenu;

touchScreenControlState touchScreen;

extern	VFileSpec	*theHousesSpecs;
extern	demoPtr		demoData;
extern	gameType	smallGame;
extern	Rect		gliderSrc[kNumGliderSrcRects];
extern	Rect		boardDestRect, boardSrcRect;
extern	Rect		localRoomsDest[];
extern	long		incrementModeTime;
extern	short		numBands, otherPlayerEscaped, demoIndex, demoHouseIndex;
extern	short		splashOriginH, splashOriginV, countDown, thisHouseIndex;
extern	short		numStarsRemaining, numChimes, saidFollow;
extern	Boolean		quitting, isMusicOn, gameOver, hasMirror, onePlayerLeft;
extern	Boolean		isPlayMusicIdle, failedMusic, quickerTransitions;
extern	Boolean		switchedOut;
extern	short		wasScoreboardTitleMode;


//==============================================================  Functions
//--------------------------------------------------------------  NewGame

void NewGame (short mode)
{
	Rect		tempRect;
	PLError_t		theErr;
	Boolean		wasPlayMusicPref;

	gameOver = false;
	theMode = kPlayMode;
	if (isPlayMusicGame)
	{
		if (!isMusicOn)
		{
			theErr = StartMusic();
			if (theErr != PLErrors::kNone)
			{
				YellowAlert(kYellowNoMusic, theErr);
				failedMusic = true;
			}
		}
		SetMusicalMode(kPlayGameScoreMode);
	}
	else
	{
		if (isMusicOn)
			StopTheMusic();
	}
	if (mode != kResumeGameMode)
		SetObjectsToDefaults();
	HideCursor();
	if (mode == kResumeGameMode)
		SetHouseToSavedRoom();
	else if (mode == kNewGameMode)
		SetHouseToFirstRoom();
	DetermineRoomOpenings();
	NilSavedMaps();

	gameFrame = 0L;
	numBands = 0;
	demoIndex = 0;
	saidFollow = 0;
	otherPlayerEscaped = kNoOneEscaped;
	onePlayerLeft = false;
	playerSuicide = false;

	if (twoPlayerGame)					// initialize glider(s)
	{
		InitGlider(&theGlider, kNewGameMode);
		InitGlider(&theGlider2, kNewGameMode);
		LoadGraphic(glidSrcMap, kGliderPictID);
		LoadGraphic(glid2SrcMap, kGlider2PictID);
	}
	else
	{
		InitGlider(&theGlider, mode);
		LoadGraphic(glidSrcMap, kGliderPictID);
		LoadGraphic(glid2SrcMap, kGliderFoilPictID);
	}

#if !BUILD_ARCADE_VERSION
//	HideMenuBarOld();		// TEMP
#endif

	DrawSurface *mainWindowSurface = mainWindow->GetDrawSurface();

	PortabilityLayer::ResolveCachingColor blackColorMain = StdColors::Black();

	tempRect = thisMac.constrainedScreen;
	tempRect.top = tempRect.bottom - 20;	// thisMac.menuHigh
	mainWindowSurface->FillRect(tempRect, blackColorMain);

#ifdef COMPILEQT
	if ((thisMac.hasQT) && (hasMovie))
	{
		theMovie.m_surface = mainWindow->GetDrawSurface();
	}
#endif

	PortabilityLayer::ResolveCachingColor blackColorWork = StdColors::Black();
	workSrcMap->FillRect(workSrcRect, blackColorWork);
//	if (quickerTransitions)
//		DissBitsChunky(&workSrcRect);
//	else
//		DissBits(&workSrcRect);

//	DebugStr("\pIf screen isn't black, exit to shell.");	// TEMP TEMP TEMP

	ResetLocale(false);
	RefreshScoreboard(kNormalTitleMode);
//	if (quickerTransitions)
//		DissBitsChunky(&justRoomsRect);
//	else
//		DissBits(&justRoomsRect);
	if (mode == kNewGameMode)
	{
		BringUpBanner();
		DumpScreenOn(&justRoomsRect, false);
	}
	else if (mode == kResumeGameMode)
	{
		DisplayStarsRemaining();
		DumpScreenOn(&justRoomsRect, false);
	}
	else
	{
		DumpScreenOn(&justRoomsRect, false);
	}

	InitGarbageRects();
	StartGliderFadingIn(&theGlider);
	if (twoPlayerGame)
	{
		StartGliderFadingIn(&theGlider2);
		TagGliderIdle(&theGlider2);
		theGlider2.dontDraw = true;
	}
	InitTelephone();
	wasPlayMusicPref = isPlayMusicGame;

#ifdef CREATEDEMODATA
	SysBeep(1);
#endif

#ifdef COMPILEQT
	if ((thisMac.hasQT) && (hasMovie) && (tvInRoom))
	{
		theMovie.m_playing = true;
		if (tvOn)
		{
			AnimationManager::GetInstance()->RegisterPlayer(&theMovie);
			AnimationManager::GetInstance()->RefreshPlayer(&theMovie);
		}
	}
#endif

	playing = true;		// everything before this line is game set-up
	PlayGame();			// everything following is after a game has ended

	ClearScoreboard();

#ifdef CREATEDEMODATA
	DumpToResEditFile((Ptr)demoData, sizeof(demoType) * (long)demoIndex);
#endif

	isPlayMusicGame = wasPlayMusicPref;
	ZeroMirrorRegion();

#ifdef COMPILEQT
	if ((thisMac.hasQT) && (hasMovie) && (tvInRoom))
	{
		tvInRoom = false;
		theMovie.m_playing = false;
	}
#endif

	twoPlayerGame = false;
	theMode = kSplashMode;
	InitCursor();
	if (isPlayMusicIdle)
	{
		if (!isMusicOn)
		{
			theErr = StartMusic();
			if (theErr != PLErrors::kNone)
			{
				YellowAlert(kYellowNoMusic, theErr);
				failedMusic = true;
			}
		}
		SetMusicalMode(kPlayWholeScoreMode);
	}
	else
	{
		if (isMusicOn)
			StopTheMusic();
	}
	NilSavedMaps();

	UpdateMenus(false);

	if (!gameOver)
	{
		RedrawSplashScreen();
	}
	WaitCommandQReleased();
	demoGoing = false;
	incrementModeTime = TickCount() + kIdleSplashTicks;
}

//--------------------------------------------------------------  DoDemoGame

void DoDemoGame (void)
{
	short		wasHouseIndex;
	Boolean		whoCares;

	wasHouseIndex = thisHouseIndex;
	whoCares = CloseHouse();
	thisHouseIndex = demoHouseIndex;
	PasStringCopy(theHousesSpecs[thisHouseIndex].m_name, thisHouseName);
	if (OpenHouse(true))
	{
		if (UseMainMenuUI())
			DismissMainMenuUI();

		demoGoing = true;
		NewGame(kNewGameMode);

		if (UseMainMenuUI())
			StartMainMenuUI();
	}
	whoCares = CloseHouse();
	thisHouseIndex = wasHouseIndex;
	PasStringCopy(theHousesSpecs[thisHouseIndex].m_name, thisHouseName);
	OpenHouse(true);
	incrementModeTime = TickCount() + kIdleSplashTicks;
	RedrawSplashScreen();
}

//--------------------------------------------------------------  InitGlider

void InitGlider (gliderPtr thisGlider, short mode)
{
	WhereDoesGliderBegin(&thisGlider->dest, mode);

	if (mode == kResumeGameMode)
		numStarsRemaining = smallGame.wasStarsLeft;
	else if (mode == kNewGameMode)
		numStarsRemaining = CountStarsInHouse();

	if (mode == kResumeGameMode)
	{
		theScore = smallGame.score;
		mortals = smallGame.numGliders;
		batteryTotal = smallGame.energy;
		bandsTotal = smallGame.bands;
		foilTotal = smallGame.foil;
		thisGlider->mode = smallGame.gliderState;
		thisGlider->facing = smallGame.facing;
		showFoil = smallGame.showFoil;

		switch (thisGlider->mode)
		{
			case kGliderBurning:
			FlagGliderBurning(thisGlider);
			break;

			default:
			FlagGliderNormal(thisGlider);
			break;
		}
	}
	else
	{
		theScore = 0L;
		mortals = kInitialGliders;
		if (twoPlayerGame)
			mortals += kInitialGliders;
		batteryTotal = 0;
		bandsTotal = 0;
		foilTotal = 0;
		thisGlider->mode = kGliderNormal;
		thisGlider->facing = kFaceRight;
		thisGlider->src = gliderSrc[0];
		thisGlider->mask = gliderSrc[0];
		showFoil = false;
	}

	QSetRect(&thisGlider->destShadow, 0, 0, kGliderWide, kShadowHigh);
	QOffsetRect(&thisGlider->destShadow, thisGlider->dest.left, kShadowTop);
	thisGlider->wholeShadow = thisGlider->destShadow;

	thisGlider->hVel = 0;
	thisGlider->vVel = 0;
	thisGlider->hDesiredVel = 0;
	thisGlider->vDesiredVel = 0;

	thisGlider->tipped = false;
	thisGlider->sliding = false;
	thisGlider->dontDraw = false;
}

//--------------------------------------------------------------  SetHouseToFirstRoom

void SetHouseToFirstRoom (void)
{
	short		firstRoom;

	firstRoom = GetFirstRoomNumber();
	ForceThisRoom(firstRoom);
}

//--------------------------------------------------------------  SetHouseToSavedRoom

void SetHouseToSavedRoom (void)
{
	ForceThisRoom(smallGame.roomNumber);
}

//--------------------------------------------------------------  HandleGameResolutionChange

void HandleGameResolutionChange(void)
{
	FlushResolutionChange();

	RecomputeInterfaceRects();
	RecreateOffscreens();
	CloseMainWindow();
	OpenMainWindow();

	if (hasMovie)
		theMovie.m_surface = mainWindow->GetDrawSurface();

	ResetLocale(true);
	InitScoreboardMap();
	RefreshScoreboard(wasScoreboardTitleMode);
	DumpScreenOn(&justRoomsRect, true);
}

//--------------------------------------------------------------  HandleTouchUp

void HandleTouchUp(touchScreenFingerID fingerID)
{
	for (int i = 0; i < touchScreenControlState::kMaxFingers; i++)
	{
		touchScreenFingerState &fstate = touchScreen.fingers[i];
		if (fstate.active && fstate.tfingerID == fingerID)
		{
			if (fstate.capturingControl == TouchScreenCtrlIDs::Menu && touchScreen.controls[fstate.capturingControl].touchRect.Contains(fstate.point))
				pendingTouchScreenMenu = true;

			fstate.active = false;
			fstate.tfingerID = touchScreenFingerID();
			fstate.capturingControl = TouchScreenCtrlIDs::Invalid;
			return;
		}
	}
}

//--------------------------------------------------------------  HandleTouchMove

void HandleTouchMove(touchScreenFingerID fingerID, const Point &pt)
{
	for (int i = 0; i < touchScreenControlState::kMaxFingers; i++)
	{
		touchScreenFingerState &fstate = touchScreen.fingers[i];

		if (fstate.active && fstate.tfingerID == fingerID)
		{
			fstate.point = pt;
			return;
		}
	}
}

//--------------------------------------------------------------  HandleTouchDown

void HandleTouchDown(touchScreenFingerID fingerID, const Point &pt)
{
	int freeFingerIndex = -1;

	for (int i = 0; i < touchScreenControlState::kMaxFingers; i++)
	{
		touchScreenFingerState &fstate = touchScreen.fingers[i];

		if (fstate.active && fstate.tfingerID == fingerID)
		{
			// Finger is already considered down, something weird happened
			HandleTouchMove(fingerID, pt);
			return;
		}
		else if (!fstate.active)
			freeFingerIndex = i;
	}

	if (freeFingerIndex < 0)
		return;

	touchScreenFingerState &fingerState = touchScreen.fingers[freeFingerIndex];

	for (int j = 0; j < TouchScreenCtrlIDs::Count; j++)
	{
		if (touchScreen.controls[j].isEnabled)
		{
			if (touchScreen.controls[j].touchRect.Contains(pt))
			{
				fingerState.tfingerID = fingerID;
				fingerState.active = true;
				fingerState.capturingControl = static_cast<TouchScreenCtrlID_t>(j);
				fingerState.point = pt;
				return;
			}
		}
	}

}

//--------------------------------------------------------------  HandleTouchLeave

void HandleTouchLeave(touchScreenFingerID fingerID)
{
	for (int i = 0; i < touchScreenControlState::kMaxFingers; i++)
	{
		touchScreenFingerState &fstate = touchScreen.fingers[i];

		if (fstate.active && fstate.tfingerID == fingerID)
		{
			fstate.tfingerID = touchScreenFingerID();
			fstate.active = false;
			fstate.capturingControl = TouchScreenCtrlIDs::Invalid;
			return;
		}
	}
}

//--------------------------------------------------------------  HandleInGameEvents

void HandleInGameEvents(void)
{
	PortabilityLayer::EventQueue *queue = PortabilityLayer::EventQueue::GetInstance();

	TimeTaggedVOSEvent evt;
	while (queue->Dequeue(&evt))
	{
		if (thisMac.isTouchscreen)
		{
			if (thisMac.isMouseTouchscreen && evt.m_vosEvent.m_eventType == GpVOSEventTypes::kMouseInput)
			{
				const GpMouseInputEvent &mouseInput = evt.m_vosEvent.m_event.m_mouseInputEvent;

				const Point mousePt = mainWindow->MouseToLocal(mouseInput);

				switch (mouseInput.m_eventType)
				{
				case GpMouseEventTypes::kDown:
					if (mouseInput.m_button == GpMouseButtons::kLeft)
						HandleTouchDown(touchScreenFingerID(), mousePt);
					break;
				case GpMouseEventTypes::kLeave:
					HandleTouchLeave(touchScreenFingerID());
					break;
				case GpMouseEventTypes::kUp:
					HandleTouchMove(touchScreenFingerID(), mousePt);
					HandleTouchUp(touchScreenFingerID());
					break;
				case GpMouseEventTypes::kMove:
					HandleTouchMove(touchScreenFingerID(), mousePt);
					break;
				default:
					break;
				};
			}

			if (!thisMac.isMouseTouchscreen && evt.m_vosEvent.m_eventType == GpVOSEventTypes::kTouchInput)
			{
				const GpTouchInputEvent &touchInput = evt.m_vosEvent.m_event.m_touchInputEvent;

				const Point touchPt = mainWindow->TouchToLocal(touchInput);

				switch (touchInput.m_eventType)
				{
				case GpTouchEventTypes::kDown:
					HandleTouchDown(touchScreenFingerID(touchInput.m_deviceID, touchInput.m_fingerID), touchPt);
					break;
				case GpTouchEventTypes::kLeave:
					HandleTouchLeave(touchScreenFingerID(touchInput.m_deviceID, touchInput.m_fingerID));
					break;
				case GpTouchEventTypes::kUp:
					HandleTouchMove(touchScreenFingerID(touchInput.m_deviceID, touchInput.m_fingerID), touchPt);
					HandleTouchUp(touchScreenFingerID(touchInput.m_deviceID, touchInput.m_fingerID));
					break;
				case GpTouchEventTypes::kMove:
					HandleTouchMove(touchScreenFingerID(), touchPt);
					break;
				default:
					break;
				};
			}
		}
	}
}

//--------------------------------------------------------------  ResetTouchScreenControlBounds

static int16_t touchScreenControlSize = 64;

void ResetTouchScreenControlBounds (void)
{
	if (!thisMac.isTouchscreen)
		return;

	const Rect centerRoomRect = localRoomsDest[kCentralRoom];

	int16_t touchScreenControlInterSpacing = 16;
	int16_t touchScreenControlEdgeSpacing = 24;

	Point points[TouchScreenCtrlIDs::Count];
	Point sizes[TouchScreenCtrlIDs::Count];

	points[TouchScreenCtrlIDs::Movement] = Point::Create(mainWindowRect.left, mainWindowRect.top);
	points[TouchScreenCtrlIDs::BatteryHelium] = Point::Create(mainWindowRect.left + touchScreenControlEdgeSpacing, mainWindowRect.top + touchScreenControlEdgeSpacing);
	points[TouchScreenCtrlIDs::Bands] = Point::Create(mainWindowRect.right - touchScreenControlEdgeSpacing - touchScreenControlSize, mainWindowRect.top + touchScreenControlEdgeSpacing);

	if (demoGoing)
		points[TouchScreenCtrlIDs::Menu] = Point::Create(mainWindowRect.right - touchScreenControlEdgeSpacing - touchScreenControlSize, mainWindowRect.top + touchScreenControlEdgeSpacing);
	else
		points[TouchScreenCtrlIDs::Menu] = Point::Create(points[TouchScreenCtrlIDs::Bands].h - touchScreenControlInterSpacing - touchScreenControlSize, points[TouchScreenCtrlIDs::BatteryHelium].v);

	for (int i = 0; i < TouchScreenCtrlIDs::Count; i++)
		sizes[i] = Point::Create(touchScreenControlSize, touchScreenControlSize);

	sizes[TouchScreenCtrlIDs::Movement] = Point::Create(mainWindowRect.Width(), mainWindowRect.Height());

	for (int i = 0; i < TouchScreenCtrlIDs::Count; i++)
	{
		Point lowerRight = points[i] + sizes[i];
		touchScreen.controls[i].graphicRect = Rect::Create(points[i].v, points[i].h, lowerRight.v, lowerRight.h);
		touchScreen.controls[i].touchRect = touchScreen.controls[i].graphicRect.Inset(-(touchScreenControlInterSpacing / 2), -(touchScreenControlInterSpacing / 2));
	}

	// Clear all active touches
	for (int i = 0; i < touchScreenControlState::kMaxFingers; i++)
	{
		touchScreen.fingers[i].tfingerID = touchScreenFingerID();
		touchScreen.fingers[i].active = false;
		touchScreen.fingers[i].capturingControl = TouchScreenCtrlIDs::Invalid;
	}
}

//--------------------------------------------------------------  InitTouchScreenControlState

void InitTouchScreenControlState(void)
{
	if (!thisMac.isTouchscreen)
		return;

	ResetTouchScreenControlBounds();

	for (int i = 0; i < touchScreenControlGraphics::Count; i++)
	{
		if (touchScreen.graphics[i] != nil)
			continue;

		int resID = touchScreenControlGraphics::kTouchScreenGraphicStartID + i;

		Rect resRect = Rect::Create(0, 0, touchScreenControlSize, touchScreenControlSize);
		(void)CreateOffScreenGWorld(&touchScreen.graphics[i], &resRect);
		LoadGraphicNoDither(touchScreen.graphics[i], resID);
	}

	pendingTouchScreenMenu = false;
}

//--------------------------------------------------------------  PlayGame

void PlayGame (void)
{
	const houseType *debugHouse = nullptr;
	if (thisHouse)
		debugHouse = *thisHouse;

	InitTouchScreenControlState();

	touchScreen.controls[TouchScreenCtrlIDs::Movement].isEnabled = (demoGoing == 0);
	touchScreen.controls[TouchScreenCtrlIDs::Bands].isEnabled = (demoGoing == 0);
	touchScreen.controls[TouchScreenCtrlIDs::BatteryHelium].isEnabled = (demoGoing == 0);
	touchScreen.controls[TouchScreenCtrlIDs::Menu].isEnabled = true;

	while ((playing) && (!quitting))
	{
		HandleInGameEvents();

		if (thisMac.isResolutionDirty)
		{
			HandleGameResolutionChange();
			ResetTouchScreenControlBounds();
		}

		gameFrame++;
		evenFrame = !evenFrame;

		if (doBackground)
		{
			Delay(2, nil);
		}

		HandleTelephone();

		if (twoPlayerGame)
		{
			HandleDynamics();
			if (!gameOver)
			{
				GetInput(&theGlider);
				GetInput(&theGlider2);
				HandleInteraction();
			}
			HandleTriggers();
			HandleBands();
			if (!gameOver)
			{
				HandleGlider(&theGlider);
				HandleGlider(&theGlider2);
			}
			if (playing)
			{
#ifdef COMPILEQT
				if ((thisMac.hasQT) && (hasMovie) && (tvInRoom) && (tvOn))
						AnimationManager::GetInstance()->RefreshPlayer(&theMovie);
#endif
				RenderFrame();
				HandleDynamicScoreboard();
			}
		}
		else
		{
			HandleDynamics();
			if (!gameOver)
			{
				if (demoGoing)
					GetDemoInput(&theGlider);
				else
					GetInput(&theGlider);
				HandleInteraction();
			}
			HandleTriggers();
			HandleBands();
			if (!gameOver)
				HandleGlider(&theGlider);
			if (playing)
			{
#ifdef COMPILEQT
				if ((thisMac.hasQT) && (hasMovie) && (tvInRoom) && (tvOn))
					AnimationManager::GetInstance()->RefreshPlayer(&theMovie);
#endif
				RenderFrame();
				HandleDynamicScoreboard();
			}
		}
		
		if (quitting) {
			DoEndGame();
		}
		
		if (gameOver)
		{
			countDown--;
			if (countDown <= 0)
			{
				HideGlider(&theGlider);
				RefreshScoreboard(kNormalTitleMode);

#if BUILD_ARCADE_VERSION
			// Need to paint over the scoreboard black.

				boardSrcMap->SetForeColor(StdColors::Black());
				boardSrcMap->FillRect(boardSrcRect);

				CopyBits((BitMap *)*GetGWorldPixMap(boardSrcMap),
						GetPortBitMapForCopyBits(GetWindowPort(boardWindow)),
						&boardSrcRect, &boardDestRect, srcCopy);

				{
					Rect		bounds;
					THandle<Picture>	thePicture;
					SInt16		hOffset;

					if (boardSrcRect.right >= 640)
						hOffset = (RectWide(&boardSrcRect) - kMaxViewWidth) / 2;
					else
						hOffset = -576;
					thePicture = GetPicture(kScoreboardPictID);
					if (!thePicture)
						RedAlert(kErrFailedGraphicLoad);
					bounds = (*thePicture)->picFrame.ToRect();
					QOffsetRect(&bounds, -bounds.left, -bounds.top);
					QOffsetRect(&bounds, hOffset, 0);
					boardSrcMap->DrawPicture(thePicture, bounds);
					thePicture.Dispose();
				}
#else
//				ShowMenuBarOld();	// TEMP
#endif

				if (mortals < 0)
					DoDiedGameOver();
				else
					DoGameOver();
			}
		}
	}

#if BUILD_ARCADE_VERSION
	{
		DrawSurface	*wasCPort = GetGraphicsPort();

		boardSrcMap->SetForeColor(StdColors::Black());
		boardSrcMap->FillRect(boardSrcRect);

		CopyBits((BitMap *)*GetGWorldPixMap(boardSrcMap),
				GetPortBitMapForCopyBits(GetWindowPort(boardWindow)),
				&boardSrcRect, &boardDestRect, srcCopy);

		SetGraphicsPort(wasCPort);
	}

	{
		Rect		bounds;
		THandle<Picture>	thePicture;
		SInt16		hOffset;

		if (boardSrcRect.right >= 640)
			hOffset = (RectWide(&boardSrcRect) - kMaxViewWidth) / 2;
		else
			hOffset = -576;
		thePicture = GetPicture(kScoreboardPictID);
		if (!thePicture)
			RedAlert(kErrFailedGraphicLoad);
		bounds = (*thePicture)->picFrame.ToRect();
		QOffsetRect(&bounds, -bounds.left, -bounds.top);
		QOffsetRect(&bounds, hOffset, 0);
		boardSrcMap->DrawPicture(thePicture, bounds);
		thePicture.Dispose();
	}

#else

//	ShowMenuBarOld();	// TEMP

#endif
}

//--------------------------------------------------------------  SetObjectsToDefaults

void SetObjectsToDefaults (void)
{
	houseType	*thisHousePtr;
	short		numRooms;
	short		r, i;
	char		wasState;
	Boolean		initState;

	thisHousePtr = *thisHouse;

	numRooms = thisHousePtr->nRooms;

	for (r = 0; r < numRooms; r++)
	{
		thisHousePtr->rooms[r].visited = false;
		for (i = 0; i < kMaxRoomObs; i++)
		{
			switch (thisHousePtr->rooms[r].objects[i].what)
			{
				case kFloorVent:
				case kCeilingVent:
				case kFloorBlower:
				case kCeilingBlower:
				case kLeftFan:
				case kRightFan:
				case kSewerGrate:
				case kInvisBlower:
				case kGrecoVent:
				case kSewerBlower:
				case kLiftArea:
				thisHousePtr->rooms[r].objects[i].data.a.state =
					thisHousePtr->rooms[r].objects[i].data.a.initial;
				break;

				case kRedClock:
				case kBlueClock:
				case kYellowClock:
				case kCuckoo:
				case kPaper:
				case kBattery:
				case kBands:
				case kGreaseRt:
				case kGreaseLf:
				case kFoil:
				case kInvisBonus:
				case kStar:
				case kSparkle:
				case kHelium:
				thisHousePtr->rooms[r].objects[i].data.c.state =
					thisHousePtr->rooms[r].objects[i].data.c.initial;
				break;

				case kDeluxeTrans:
				initState = (thisHousePtr->rooms[r].objects[i].data.d.wide & 0xF0) >> 4;
				thisHousePtr->rooms[r].objects[i].data.d.wide &= 0xF0;
				thisHousePtr->rooms[r].objects[i].data.d.wide += initState;
				break;

				case kCeilingLight:
				case kLightBulb:
				case kTableLamp:
				case kHipLamp:
				case kDecoLamp:
				case kFlourescent:
				case kTrackLight:
				case kInvisLight:
				thisHousePtr->rooms[r].objects[i].data.f.state =
					thisHousePtr->rooms[r].objects[i].data.f.initial;
				break;

				case kStereo:
				thisHousePtr->rooms[r].objects[i].data.g.state = isPlayMusicGame;
				break;

				case kShredder:
				case kToaster:
				case kMacPlus:
				case kGuitar:
				case kTV:
				case kCoffee:
				case kOutlet:
				case kVCR:
				case kMicrowave:
				thisHousePtr->rooms[r].objects[i].data.g.state =
					thisHousePtr->rooms[r].objects[i].data.g.initial;
				break;

				case kBalloon:
				case kCopterLf:
				case kCopterRt:
				case kDartLf:
				case kDartRt:
				case kBall:
				case kDrip:
				case kFish:
				thisHousePtr->rooms[r].objects[i].data.h.state =
					thisHousePtr->rooms[r].objects[i].data.h.initial;
				break;

			}
		}
	}
}

//--------------------------------------------------------------  HideGlider

void HideGlider (gliderPtr thisGlider)
{
	Rect		tempRect;

	tempRect = thisGlider->whole;
	QOffsetRect(&tempRect, playOriginH, playOriginV);
	CopyRectWorkToMain(&tempRect);

	if (hasMirror)
	{
		QOffsetRect(&tempRect, -20, -16);
		CopyRectWorkToMain(&tempRect);
	}

	tempRect = thisGlider->wholeShadow;
	QOffsetRect(&tempRect, playOriginH, playOriginV);
	CopyRectWorkToMain(&tempRect);
}

//--------------------------------------------------------------  InitTelephone

void InitTelephone (void)
{
	thePhone.nextRing = RandomInt(kRingSpread) + kRingBaseDelay;
	thePhone.rings = RandomInt(3) + 3;
	thePhone.delay = kRingDelay;

	theChimes.nextRing = RandomInt(kChimeDelay) + 1;
}

//--------------------------------------------------------------  HandleTelephone

void HandleTelephone (void)
{
	short		delayTime;

	if (!phoneBitSet)
	{
		if (thePhone.nextRing == 0)
		{
			if (thePhone.delay == 0)
			{
				thePhone.delay = kRingDelay;
				PlayPrioritySound(kPhoneRingSound, kPhoneRingPriority);
				thePhone.rings--;
				if (thePhone.rings == 0)
				{
					thePhone.nextRing = RandomInt(kRingSpread) + kRingBaseDelay;
					thePhone.rings = RandomInt(3) + 3;
				}
			}
			else
				thePhone.delay--;
		}
		else
			thePhone.nextRing--;
	}
	// handle also the wind chimes (if they are present)

	if (numChimes > 0)
	{
		if (theChimes.nextRing == 0)
		{
			if (RandomInt(2) == 0)
				PlayPrioritySound(kChime1Sound, kChime1Priority);
			else
				PlayPrioritySound(kChime2Sound, kChime2Priority);

			delayTime = kChimeDelay / numChimes;
			if (delayTime < 2)
				delayTime = 2;

			theChimes.nextRing = RandomInt(delayTime) + 1;
		}
		else
			theChimes.nextRing--;
	}
}

//--------------------------------------------------------------  StrikeChime

void StrikeChime (void)
{
	theChimes.nextRing = 0;
}

//--------------------------------------------------------------  RestoreEntireGameScreen

void RestoreEntireGameScreen (void)
{
	Rect		tempRect;

	HideCursor();

#if !BUILD_ARCADE_VERSION
//	HideMenuBarOld();		// TEMP
#endif

	DrawSurface *surface = mainWindow->GetDrawSurface();
	tempRect = thisMac.constrainedScreen;

	PortabilityLayer::ResolveCachingColor blackColor = StdColors::Black();
	surface->FillRect(tempRect, blackColor);

	ResetLocale(false);
	RefreshScoreboard(kNormalTitleMode);
//	if (quickerTransitions)
//		DissBitsChunky(&justRoomsRect);
//	else
//		DissBits(&justRoomsRect);
}

