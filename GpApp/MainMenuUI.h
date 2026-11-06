#pragma once

struct Point;
struct Window;

void StartMainMenuUI();
void TickMainMenuUI();
void DismissMainMenuUI();
void HandleMainMenuUIResolutionChange();
bool HandleMainMenuUIClick(Window *window, const Point &pt);

// The big-button main menu: on touchscreens, and in the terminal build,
// where the Mac menu bar is too small to read (Asciiverse).
void SetUseMainMenuUI(bool useMainMenuUI);
bool UseMainMenuUI();
