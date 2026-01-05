# Vial support - DISABLED to enable custom tap dance functions for layer switching
# VIA_ENABLE = yes
# VIAL_ENABLE = yes
# VIALRGB_ENABLE = yes

# Core features
MOUSEKEY_ENABLE = yes
EXTRAKEY_ENABLE = yes
# AUTO_SHIFT_ENABLE = yes  # Disabled - causes held keys to become capitals
RAW_ENABLE = yes
CAPS_WORD_ENABLE = yes
TAP_DANCE_ENABLE = yes
KEY_OVERRIDE_ENABLE = yes
# COMBO_ENABLE = yes  # Disabled - no combos defined

# Custom features
SRC += yxa_features.c
