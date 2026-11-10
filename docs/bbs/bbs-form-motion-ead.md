# BBS EAD events of kh1-combo's form motions

From each character's `arc/effect/p0X_<who>/p0Xex_000.ead`, read 2026-11-08, for the motions in kh1-combo's `forms.rs` `fn moveset`. A motion id is style x 1000 + the motion's name (3402 = name `402`); the EAD keys by name alone.

## How to read the tables

- **Frame:** 30 a second (the file's u16 halved; ESTIMATE, as DDD's).
- **End:** 0 = none.
- **Group:** the effect group. An event runs when its group is negative or equals the motion's group.
  - A style's own motion runs with group 400 + its style (3402 → 403).
  - A base motion (style 0) runs with the style set by `kh1_flowmotion_bbs_style`. With none set, only events with a negative group run.
- **Mode:** flags & 3, the DDD rule for leaving the motion: 2 = a looping effect keeps going after the motion is left; anything else stops it.
- **Follow:** flag 4, the effect follows its bone every frame.
- **End stops:** flag 0x10, the effect stops at its end frame.
- **Loaded:** only events whose file kh1-flowmotion loads play (pst_4NN, p00_com, p0X_<who>). The others are marked *not loaded*.

**HIT elements:** none of these motions' EADs start a HIT element. BBS fires hits from its attack data, so keep your per-foe HIT0/HIT1.

## Ventus 3402 (name 402)
No events.

## Ventus 3403 (name 403)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 11.5 | 17.5 | p_st_com_00 | P_ST_COM_MLC0_0 | Root | 403 | 0xc | 0 | yes | yes |
| 2 | 0 | p_st_com_00 | P_ST_COM_MLC0_0 | Root | 403 | 0xc | 0 | yes | yes |

## Ventus 3404 (name 404)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 14 | 30.5 | p_ex_bas_00 | P_MO_DSH_FLO0_0 | Root | 403 | 0xc | 0 | yes | yes |
| 14 | 27.5 | p_st_03_spr | P_ST_SPR_DSH0_0 | Root | 403 | 0xc | 0 | yes | yes |
| 0 | 10 | p_st_com_00 | P_ST_COM_TAM0_0 | mune | 403 | 0xc | 0 | yes | yes |

## Ventus 3405 (name 405)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 30 | 27.5 | p_st_03_spr | P_ST_SPR_DSH0_0 | Root | 403 | 0xc | 0 | yes | yes |
| 0 | 0 | p_st_03_spr | P_ST_SPR_ORA1_0 | mune | 403 | 0x1d | 1 | yes | yes |
| 67 | 0 | p_st_03_spr | P_ST_SPR_ZHA0_0 | Root | 403 | 0x8 | 0 | no | yes |

## Ventus 7420 (name 420)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 7 | 0 | p_st_09_cyc | P_ST_CYC_MLC0_0 | Root | 409 | 0xc | 0 | yes | yes |
| 1 | 0 | p_st_09_cyc | P_ST_CYC_MLC0_0 | Root | 409 | 0xc | 0 | yes | yes |

## Ventus 7421 (name 421)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 22 | 0 | p_ra_com_00 | P_RA_COM_CUH0_0 | R_buki | 409 | 0xc | 0 | yes | *not loaded* |

## Ventus 7422 (name 422)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 6 | 0 | p_st_09_cyc | P_ST_CYC_MLC0_0 | Root | 409 | 0xc | 0 | yes | yes |
| 1 | 0 | p_st_09_cyc | P_ST_CYC_MLC0_0 | Root | 409 | 0xc | 0 | yes | yes |

## Ventus 7423 (name 423)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 51 | 0 | p_ra_com_00 | P_RA_COM_CUH0_0 | R_buki | 409 | 0xc | 0 | yes | *not loaded* |
| 0 | 0 | p_st_09_cyc | P_ST_CYC_ROL0_0 | Root | 409 | 0xc | 0 | yes | yes |

## Ventus 9435 (name 435)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | p_st_14_skr | P_ST_SKR_ORA1_0 | mune | 414 | 0x0 | 0 | no | yes |
| 21 | 32.5 | p_st_com_00 | P_ST_COM_TAM0_0 | mune | 414 | 0x1d | 1 | yes | yes |

## Ventus 9436 (name 436)
No events.

## Ventus 9437 (name 437)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | p_st_14_skr | P_ST_SKR_ORA1_0 | mune | 414 | 0x0 | 0 | no | yes |
| 22 | 40 | p_st_14_skr | P_ST_SKR_DSH1_0 | Root | 414 | 0x14 | 0 | yes | yes |
| 42 | 0 | p_st_14_skr | P_ST_SKR_HAN1_0 | Root | 414 | 0x0 | 0 | no | yes |
| 42 | 0 | p_st_14_skr | P_ST_SKR_HAN2_0 | Root | 414 | 0x3000 | 0 | no | yes |

## Ventus 9504 (name 504)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 5 | 18 | p_ss_blb_00 | P_SS_BLB_CAL0_0 | mune | 533 | 0x11 | 1 | no | *not loaded* |
| 17.5 | 0 | p_dl_ki1_00 | P_DL_KI1_ORA0_0 | Root | 602 | 0x6 | 2 | yes | *not loaded* |
| 1 | 11 | p_st_com_00 | P_ST_COM_TAM0_0 | mune | 605 | 0x15 | 1 | yes | yes |
| 11 | 0 | p_st_com_00 | P_ST_COM_ORA2_0 | mune | 605 | 0x14c | 0 | yes | yes |
| 1 | 11 | p_st_com_00 | P_ST_COM_TAM0_0 | mune | 606 | 0x15 | 1 | yes | yes |
| 11 | 0 | p_st_com_00 | P_ST_COM_ORA2_0 | mune | 606 | 0x14c | 0 | yes | yes |
| 0 | 0 | p_nt_vlt_00 | P_NT_VLT_HOB0_0 | Root | 709 | 0x6 | 2 | yes | *not loaded* |
| 7 | 13 | p_st_com_00 | P_ST_COM_ORA2_0 | mune | 709 | 0x14c | 0 | yes | yes |

## Ventus 9608 (name 608)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | p_st_14_skr | P_ST_SKR_LOC2_0 | Root | 414 | 0x4 | 0 | yes | yes |
| 6.5 | 0 | p_st_20_rmr | P_ST_RMR_FIR0_0 | Root | 420 | 0x4c | 0 | yes | yes |
| 10.5 | 0 | p_st_20_rmr | P_ST_RMR_FIR0_0 | Root | 420 | 0x4c | 0 | yes | yes |
| 14.5 | 0 | p_st_20_rmr | P_ST_RMR_FIR0_0 | Root | 420 | 0x4c | 0 | yes | yes |
| 2.5 | 0 | p_st_20_rmr | P_ST_RMR_FIR0_0 | Root | 420 | 0x4d | 1 | yes | yes |
| 9 | 0 | p_st_20_rmr | P_ST_RMR_BLT0_0 | Root | 420 | 0x4f | 3 | yes | yes |
| 13 | 0 | p_st_20_rmr | P_ST_RMR_BLT0_0 | Root | 420 | 0x4f | 3 | yes | yes |
| 17 | 0 | p_st_20_rmr | P_ST_RMR_BLT0_0 | Root | 420 | 0x4f | 3 | yes | yes |
| 5 | 0 | p_st_20_rmr | P_ST_RMR_BLT0_0 | Root | 420 | 0x4f | 3 | yes | yes |
| 0 | 0 | p_st_20_rmr | P_ST_RMR_HAN0_0 | Root | 420 | 0x4c | 0 | yes | yes |
| 10 | 0 | p_st_20_rmr | P_ST_RMR_BLT0_0 | Root | 420 | 0x4f | 3 | yes | yes |
| 0 | 0 | p_dl_ki0_00 | P_DL_KI0_LOC0_0 | Root | 601 | 0x4 | 0 | yes | *not loaded* |
| 0 | 0 | p_dl_ki1_00 | P_DL_KI1_LOC0_0 | Root | 602 | 0x4 | 0 | yes | *not loaded* |

## Ventus 12481 (name 481)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 4 | 0 | p_st_20_rmr | P_ST_RMR_KIR1_0 | Root | 420 | 0x8 | 0 | no | yes |

## Ventus 12482 (name 482)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 6 | 0 | p_st_20_rmr | P_ST_RMR_KIR1_0 | Root | 420 | 0x8 | 0 | no | yes |
| 1.5 | 0 | p_st_20_rmr | P_ST_RMR_KIR1_0 | Root | 420 | 0x8 | 0 | no | yes |

## Ventus 12483 (name 483)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 13 | 0 | p_st_20_rmr | P_ST_RMR_KIR2_0 | Root | 420 | 0x48 | 0 | no | yes |
| 6.5 | 0 | p_st_20_rmr | P_ST_RMR_KIR2_0 | Root | 420 | 0x48 | 0 | no | yes |
| 18.5 | 0 | p_st_20_rmr | P_ST_RMR_KIR2_0 | Root | 420 | 0x48 | 0 | no | yes |
| 0 | 0 | p_st_20_rmr | P_ST_RMR_KIR2_0 | Root | 420 | 0x48 | 0 | no | yes |

## Ventus 14430 (name 430)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 25 | p_st_18_wib | P_ST_WIB_APP3_0 | L_buki | 418 | 0x1c | 0 | yes | yes |
| 0 | 25 | p_st_18_wib | P_ST_WIB_APP3_0 | R_buki | 418 | 0x1c | 0 | yes | yes |

## Ventus 14431 (name 431)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 26.5 | 0 | p_ra_com_00 | P_RA_COM_CUH0_0 | R_te | 418 | 0x4 | 0 | yes | *not loaded* |
| 25 | 0 | p_ra_com_00 | P_RA_COM_WRP0_0 | L_buki | 418 | 0x10 | 0 | no | *not loaded* |
| 26 | 0 | p_ra_com_00 | P_RA_COM_WRP0_0 | R_buki | 418 | 0x10 | 0 | no | *not loaded* |
| 0 | 24 | p_st_18_wib | P_ST_WIB_APP3_0 | L_buki | 418 | 0xc | 0 | yes | yes |
| 0 | 26 | p_st_18_wib | P_ST_WIB_APP3_0 | R_buki | 418 | 0xc | 0 | yes | yes |

## Ventus 14432 (name 432)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 25 | p_st_18_wib | P_ST_WIB_APP3_0 | L_buki | 418 | 0x1c | 0 | yes | yes |
| 0 | 25 | p_st_18_wib | P_ST_WIB_APP3_0 | R_buki | 418 | 0x1c | 0 | yes | yes |

## Ventus 14433 (name 433)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 40 | 50 | p_st_18_wib | P_ST_WIB_APP3_0 | R_buki | 418 | 0x1c | 0 | yes | yes |
| 13 | 30 | p_st_18_wib | P_ST_WIB_DSH0_0 | Root | 418 | 0xc | 0 | yes | yes |
| 0.5 | 12 | p_st_com_00 | P_ST_COM_TAM0_0 | mune | 418 | 0x1c | 0 | yes | yes |

## Ventus 14434 (name 434)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 80 | 0 | p_st_18_wib | P_ST_WIB_HAN0_0 | Root | 418 | 0x8 | 0 | no | yes |
| 10 | 0 | p_st_18_wib | P_ST_WIB_ORA1_0 | mune | 418 | 0x4 | 0 | yes | yes |
| 20 | 39 | p_st_com_00 | P_ST_COM_TAM0_0 | mune | 418 | 0x1c | 0 | yes | yes |
| 60 | 75 | p_st_com_00 | P_ST_COM_TAM0_0 | mune | 418 | 0x1c | 0 | yes | yes |

## Aqua 2401 (name 401)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 13.5 | 15 | p_st_com_00 | P_ST_COM_MLC0_0 | Root | 402 | 0xc | 0 | yes | yes |
| 26 | 0 | p_ra_com_00 | P_RA_COM_CUH0_0 | R_te | 402 | 0xc | 0 | yes | *not loaded* |

## Aqua 2402 (name 402)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 10 | 12.5 | p_st_com_00 | P_ST_COM_MLC0_0 | Root | 402 | 0xc | 0 | yes | yes |
| 29.5 | 0 | p_ra_com_00 | P_RA_COM_CUH0_0 | R_te | 402 | 0xc | 0 | yes | *not loaded* |

## Aqua 2403 (name 403)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 10 | 23.5 | p_st_02_mag | P_ST_MAG_KEY1_0 | Root | 402 | 0x1c | 0 | yes | yes |
| 10 | 15 | p_ex_bas_00 | P_MO_DSH_FLO0_0 | Root | 402 | 0xc | 0 | yes | yes |
| 33.5 | 0 | p_ra_com_00 | P_RA_COM_CUH0_0 | R_te | 402 | 0xc | 0 | yes | *not loaded* |

## Aqua 2404 (name 404)
No events.

## Aqua 4413 (name 413)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 10 | 13 | p_st_04_fib | P_ST_FIB_BLT0_0 | kubi | 404 | 0x4 | 0 | yes | yes |
| 8 | 10 | p_st_04_fib | P_ST_FIB_BLT0_0 | kubi | 404 | 0x4 | 0 | yes | yes |
| 10 | 0 | p_st_05_dmd | P_ST_DMD_MLC0_0 | Root | 405 | 0x8 | 0 | no | yes |
| 2.5 | 12.5 | s_fb_pc_00 | FB_PC_ATR_0 | R_te | 820 | 0x44 | 0 | yes | *not loaded* |

## Aqua 4414 (name 414)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 15 | 0 | p_st_04_fib | P_ST_FIB_HAN0_0 | mune | 404 | 0x4 | 0 | yes | yes |
| 0 | 0 | p_st_05_dmd | P_ST_DMD_ICE1_0 | mune | 405 | 0x4 | 0 | yes | yes |
| 0 | 0 | p_st_05_dmd | P_ST_DMD_ICE0_0 | Root | 405 | 0x4 | 0 | yes | yes |
| 12 | 0 | p_st_14_skr | P_ST_SKR_HAN0_0 | mune | 414 | 0x4 | 0 | yes | yes |

## Aqua 4415 (name 415)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 13.5 | 38 | p_st_com_00 | P_ST_COM_TAM0_0 | mune | 404 | 0x14 | 0 | yes | yes |
| 45 | 50 | p_st_com_00 | P_ST_COM_MLC0_0 | Root | 404 | 0xe | 2 | yes | yes |
| 49 | 54 | p_st_com_00 | P_ST_COM_MLC0_0 | Root | 404 | 0xe | 2 | yes | yes |
| 0.5 | 0 | p_st_04_fib | P_ST_FIB_ORA1_0 | mune | 404 | 0x6 | 2 | yes | yes |
| 45 | 55 | p_st_04_fib | P_ST_FIB_BLT0_0 | Root | 404 | 0x4 | 0 | yes | yes |

## Aqua 5412 (name 412)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 15 | 0 | p_st_04_fib | P_ST_FIB_HAN0_0 | mune | 404 | 0x4 | 0 | yes | yes |
| 0 | 0 | p_st_05_dmd | P_ST_DMD_ICE1_0 | mune | 405 | 0x4 | 0 | yes | yes |
| 0 | 0 | p_st_05_dmd | P_ST_DMD_ICE0_0 | Root | 405 | 0x4 | 0 | yes | yes |

## Aqua 6418 (name 418)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | p_st_06_thd | P_ST_THD_ORA1_0 | mune | 406 | 0x1c | 0 | yes | yes |
| 21 | 64.5 | p_st_06_thd | P_ST_THD_HAN2_0 | Root | 406 | 0x1d | 1 | yes | yes |
| 21 | 67.5 | p_st_06_thd | P_ST_THD_HAN1_0 | Root | 406 | 0x4c | 0 | yes | yes |
| 3.5 | 0 | p_st_06_thd | P_ST_THD_HAN0_0 | Root | 406 | 0xc | 0 | yes | yes |

## Aqua 8419 (name 419)
No events.

## Aqua 8420 (name 420)
No events.

## Aqua 8421 (name 421)
No events.

## Aqua 8422 (name 422)
No events.

## Aqua 9436 (name 436)
No events.

## Aqua 9437 (name 437)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 | p_st_14_skr | P_ST_SKR_ORA1_0 | mune | 414 | 0x0 | 0 | no | yes |
| 42 | 0 | p_st_14_skr | P_ST_SKR_HAN2_0 | Root | 414 | 0x3000 | 0 | no | yes |
| 42 | 0 | p_st_14_skr | P_ST_SKR_HAN1_0 | Root | 414 | 0x0 | 0 | no | yes |
| 22 | 40 | p_st_14_skr | P_ST_SKR_DSH1_0 | Root | 414 | 0x14 | 0 | yes | yes |

## Aqua 9504 (name 504)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 15.5 | 0 | p_dl_ki1_00 | P_DL_KI1_ORA0_0 | Root | 602 | 0x6 | 2 | yes | *not loaded* |
| 1 | 11 | p_st_com_00 | P_ST_COM_TAM0_0 | mune | 605 | 0x15 | 1 | yes | yes |
| 11 | 0 | p_st_com_00 | P_ST_COM_ORA2_0 | mune | 605 | 0x14c | 0 | yes | yes |
| 1 | 11 | p_st_com_00 | P_ST_COM_TAM0_0 | mune | 606 | 0x15 | 1 | yes | yes |
| 11 | 0 | p_st_com_00 | P_ST_COM_ORA2_0 | mune | 606 | 0x14c | 0 | yes | yes |
| 7 | 13 | p_st_com_00 | P_ST_COM_ORA2_0 | mune | 709 | 0x14c | 0 | yes | yes |
| 0 | 0 | p_nt_vlt_00 | P_NT_VLT_HOB0_0 | Root | 709 | 0x6 | 2 | yes | *not loaded* |

## Aqua 11451 (name 451)
No events.

## Aqua 11452 (name 452)
No events.

## Aqua 11453 (name 453)
No events.

## Aqua 11454 (name 454)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 5 | 0 | p_st_com_00 | P_ST_COM_TAM0_0 | mune | 413 | 0x15 | 1 | yes | yes |
| 5 | 0 | p_st_13_gsd | P_ST_GSD_ORA1_0 | mune | 413 | 0x115 | 1 | yes | yes |

## Aqua 11455 (name 455)
No events.

## Aqua 12481 (name 481)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 0 | p_st_20_rmr | P_ST_RMR_KIR1_0 | Root | 420 | 0x48 | 0 | no | yes |

## Aqua 12482 (name 482)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 7.5 | 0 | p_st_20_rmr | P_ST_RMR_KIR1_0 | Root | 420 | 0x48 | 0 | no | yes |
| 0 | 0 | p_st_20_rmr | P_ST_RMR_KIR1_0 | Root | 420 | 0x48 | 0 | no | yes |

## Terra 1405 (name 405)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 6 | 10 | p_ex_bas_00 | P_MO_DSH_FLO0_0 | Root | 0 | 0xc | 0 | yes | yes |
| 6 | 10 | p_ex_bas_00 | P_MO_DSH_FLO0_0 | Root | 401 | 0xc | 0 | yes | yes |
| 6 | 10 | p_ex_bas_00 | P_MO_DSH_FLO0_0 | Root | 404 | 0xc | 0 | yes | yes |
| 6 | 10 | p_ex_bas_00 | P_MO_DSH_FLO0_0 | Root | 405 | 0xc | 0 | yes | yes |
| 6 | 10 | p_ex_bas_00 | P_MO_DSH_FLO0_0 | Root | 406 | 0xc | 0 | yes | yes |
| 6 | 10 | p_ex_bas_00 | P_MO_DSH_FLO0_0 | Root | 415 | 0xc | 0 | yes | yes |
| 10 | 11 | p_dl_pe0_00 | P_DL_PE0_TUK1_0 | Root | 625 | 0xc | 0 | yes | *not loaded* |
| 1 | 0 | p_st_com_00 | P_ST_COM_ORA2_0 | mune | 625 | 0x14c | 0 | yes | yes |

## Terra 1407 (name 407)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 12.5 | 35 | p_ju_cat_00 | P_JU_CAT_TAM0_0 | mune | 314 | 0x14 | 0 | yes | *not loaded* |
| 53 | 0 | p_ju_cat_00 | P_JU_CAT_ZHA0_0 | Root | 314 | 0x8 | 0 | no | *not loaded* |
| 53 | 0 | p_ju_cat_00 | P_JU_CAT_ZMN0_0 | Root | 314 | 0x8 | 0 | no | *not loaded* |
| 0.5 | 1 | p_st_01_fam | P_ST_FAM_ORA1_0 | mune | 401 | 0x1c | 0 | yes | yes |
| 12.5 | 30 | p_st_com_00 | P_ST_COM_TAM0_0 | mune | 401 | 0x1c | 0 | yes | yes |
| 53 | 0 | p_st_com_00 | P_ST_COM_ZHA0_0 | Root | 401 | 0x8 | 0 | no | yes |
| 53 | 0 | p_st_01_fam | P_ST_FAM_ZHT0_0 | Root | 401 | 0x8 | 0 | no | yes |

## Terra 8419 (name 419)
No events.

## Terra 8420 (name 420)
No events.

## Terra 8421 (name 421)
No events.

## Terra 10370 (name 370)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 4 | 0 | m_hi_frz_00 | M_HI_FRZ_CAL0_0 | mune | 165 | 0x5 | 1 | yes | *not loaded* |
| 0 | 0 | p_cr_bfo_00 | P_CR_BFO_HAN0_0 | Root | 380 | 0x0 | 0 | no | *not loaded* |
| 6 | 0 | p_cr_bfo_00 | P_CR_BFO_TAM0_0 | mune | 380 | 0x4 | 0 | yes | *not loaded* |
| 5 | 0 | p_cr_mgs_00 | P_CR_MGS_CAL0_0 | mune | 381 | 0x5 | 1 | yes | *not loaded* |
| 0 | 0 | p_cr_kmi_00 | P_CR_KMI_TAM0_0 | mune | 383 | 0x1c | 0 | yes | *not loaded* |
| 0 | 0 | p_cr_kks_00 | P_CR_KKS_TAM0_0 | mune | 384 | 0x1c | 0 | yes | *not loaded* |
| 0.5 | 0 | p_st_12_rob | P_ST_ROB_ORA1_0 | mune | 412 | 0xe | 2 | yes | yes |
| 5 | 0 | p_nt_rgc_00 | P_NT_RGC_TAM2_0 | Root | 708 | 0x44 | 0 | yes | *not loaded* |
| 0 | 0 | p_nt_rgc_00 | P_NT_RGC_SHU0_0 | Root | 708 | 0x44 | 0 | yes | *not loaded* |
| 5 | 0 | p_st_com_00 | P_ST_COM_ORA2_0 | mune | 708 | 0x14c | 0 | yes | yes |
| 4 | 0 | p_et_dct_00 | P_ET_DCT_CAL0_0 | mune | 755 | 0x5 | 1 | yes | *not loaded* |
| 2 | 0 | p_et_fs1_00 | P_ET_FS1_CAL0_0 | mune | 867 | 0x5 | 1 | yes | *not loaded* |
| 0 | 44 | p_st_com_00 | P_ST_COM_TAM0_0 | hara | 878 | 0x41c | 0 | yes | yes |
| 0 | 0 | p_cr_kks_00 | P_CR_KKS_TAM1_0 | mune | 1384 | 0x1c | 0 | yes | *not loaded* |

## Terra 10371 (name 371)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 10 | 0 | m_hi_frz_00 | M_HI_FRZ_FIR0_0 | Root | 165 | 0x0 | 0 | no | *not loaded* |
| 10 | 0 | m_hi_frz_00 | M_HI_FRZ_FIR1_0 | Root | 165 | 0x0 | 0 | no | *not loaded* |
| 0 | 0 | p_cr_bfo_00 | P_CR_BFO_SHO0_0 | Root | 380 | 0xc | 0 | yes | *not loaded* |
| 0 | 0 | p_cr_mgs_00 | P_CR_MGS_FIR0_0 | Root | 381 | 0x0 | 0 | no | *not loaded* |
| 0 | 0 | p_cr_kmi_00 | P_CR_KMI_SHO0_0 | Root | 383 | 0x0 | 0 | no | *not loaded* |
| 7 | 0 | p_cr_kks_00 | P_CR_KKS_JMN0_0 | Root | 384 | 0x0 | 0 | no | *not loaded* |
| 7 | 0 | p_cr_kks_00 | P_CR_KKS_KAZ0_0 | Root | 384 | 0x0 | 0 | no | *not loaded* |
| 7 | 0 | p_cr_kks_00 | P_CR_KKS_SHO0_0 | Root | 384 | 0x0 | 0 | no | *not loaded* |
| 11 | 0 | p_st_12_rob | P_ST_ROB_RHA0_0 | Root | 412 | 0x8 | 0 | no | yes |
| 7 | 0 | p_et_dct_00 | P_ET_DCT_LOC0_0 | Root | 755 | 0x4c | 0 | yes | *not loaded* |
| 5 | 0 | p_cl_bsl_00 | P_CL_BSL_MLC0_0 | Root | 867 | 0xc | 0 | yes | *not loaded* |
| 13 | 0 | p_cl_bsl_00 | P_CL_BSL_MLC0_0 | Root | 867 | 0xc | 0 | yes | *not loaded* |
| 21 | 0 | p_cl_bsl_00 | P_CL_BSL_MLC0_0 | Root | 867 | 0xc | 0 | yes | *not loaded* |
| 23 | 0 | p_et_fa0_00 | P_ET_FA0_FIR0_0 | hara | 878 | 0x4 | 0 | yes | *not loaded* |
| 20 | 0 | p_et_fa0_00 | P_ET_FA0_KAZ0_0 | Root | 878 | 0xc | 0 | yes | *not loaded* |
| 20 | 0 | p_et_fa4_00 | P_ET_FA4_TAM0_0 | Root | 878 | 0xc | 0 | yes | *not loaded* |
| 20 | 0 | p_et_fa4_00 | P_ET_FA4_TAM0_0 | Root | 878 | 0xc | 0 | yes | *not loaded* |
| 20 | 0 | p_et_fa4_00 | P_ET_FA4_TAM0_0 | Root | 878 | 0xc | 0 | yes | *not loaded* |
| 20 | 0 | p_et_fa4_00 | P_ET_FA4_TAM0_0 | Root | 878 | 0xc | 0 | yes | *not loaded* |
| 7 | 0 | p_cr_kks_00 | P_CR_KKS_JMN1_0 | Root | 1384 | 0x0 | 0 | no | *not loaded* |
| 7 | 0 | p_cr_kks_00 | P_CR_KKS_KAZ0_0 | Root | 1384 | 0x0 | 0 | no | *not loaded* |
| 7 | 0 | p_cr_kks_00 | P_CR_KKS_SHO1_0 | Root | 1384 | 0x0 | 0 | no | *not loaded* |

## Terra 10450 (name 450)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 15.5 | 0 | p_st_12_rob | P_ST_ROB_ZHT0_0 | Root | 412 | 0x8 | 0 | no | yes |

## Terra 10451 (name 451)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 2.5 | 0 | p_st_12_rob | P_ST_ROB_ZWK0_0 | Root | 412 | 0x8 | 0 | no | yes |

## Terra 10452 (name 452)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 2.5 | 0 | p_st_12_rob | P_ST_ROB_ZWK0_0 | Root | 412 | 0x8 | 0 | no | yes |

## Terra 10453 (name 453)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 11 | 0 | p_st_12_rob | P_ST_ROB_ZHT0_0 | Root | 412 | 0x8 | 0 | no | yes |
| 32 | 0 | p_st_12_rob | P_ST_ROB_ZHT1_0 | Root | 412 | 0x8 | 0 | no | yes |

## Terra 10454 (name 454)
No events.

## Terra 10455 (name 455)
No events.

## Terra 12481 (name 481)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 5 | 0 | p_st_20_rmr | P_ST_RMR_KIR1_0 | Root | 420 | 0x48 | 0 | no | yes |

## Terra 12482 (name 482)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 7.5 | 0 | p_st_20_rmr | P_ST_RMR_KIR1_0 | Root | 420 | 0x48 | 0 | no | yes |
| 0 | 0 | p_st_20_rmr | P_ST_RMR_KIR1_0 | Root | 420 | 0x48 | 0 | no | yes |

## Terra 13459 (name 459)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 6 | 12 | p_st_15_dim | P_ST_DIM_LOC0_0 | L_te | 415 | 0x4 | 0 | yes | yes |

## Terra 13460 (name 460)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 10 | 15 | p_st_15_dim | P_ST_DIM_LOC0_0 | L_te | 415 | 0x4 | 0 | yes | yes |
| 26 | 34 | p_st_15_dim | P_ST_DIM_LOC0_0 | L_te | 415 | 0x4 | 0 | yes | yes |

## Terra 13461 (name 461)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 22 | 34 | p_st_com_00 | P_ST_COM_MLC0_0 | Root | 415 | 0x4 | 0 | yes | yes |
| 26 | 34 | p_st_com_00 | P_ST_COM_MLC0_0 | Root | 415 | 0x4 | 0 | yes | yes |
| 12 | 17.5 | p_st_15_dim | P_ST_DIM_LOC0_0 | L_te | 415 | 0x4 | 0 | yes | yes |
| 27 | 34 | p_st_15_dim | P_ST_DIM_LOC0_0 | L_te | 415 | 0x4 | 0 | yes | yes |

## Terra 13463 (name 463)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 9 | 0 | p_st_15_dim | P_ST_DIM_ORA1_0 | mune | 415 | 0x0 | 0 | no | yes |
| 21 | 0 | p_st_15_dim | P_ST_DIM_HID0_0 | Root | 415 | 0xc | 0 | yes | yes |
| 11 | 0 | p_st_15_dim | P_ST_DIM_IN00_0 | Root | 415 | 0xc | 0 | yes | yes |

## Terra 13464 (name 464)
| Frame | End | File | Element | Bone | Group | Flags | Mode | Follow | Loaded |
|---|---|---|---|---|---|---|---|---|---|
| 7 | 25 | p_st_15_dim | P_ST_DIM_DHD0_0 | L_ude2 | 415 | 0x1c | 0 | yes | yes |
| 0 | 0 | p_st_15_dim | P_ST_DIM_OUT0_0 | Root | 415 | 0xc | 0 | yes | yes |

## Style change: motion 408

Each character's 408 holds one block per style, by group (400 + the style).

- The block's style ORA0 (and ORA2) go on mune, and p_st_com_00's ORA0/ORA1 on the limbs, hips and head.
- Their flags are 0x10e: mode 2 (they outlive the motion), they follow their bones, and they turn with them.
- CAL1 (0x5) is the entry flash.
- `kh1_flowmotion_bbs_motion_fx(chara, 408, group, 1)` starts one block and keeps its loops until `kh1_flowmotion_bbs_fx_stop`.

The four forms:

| Form | Call |
|---|---|
| Gale (Fever Pitch) | `kh1_flowmotion_bbs_motion_fx(1, 408, 403, 1)` |
| Mystic (Spellweaver) | `kh1_flowmotion_bbs_motion_fx(2, 408, 402, 1)` |
| Anti (Dark Impulse, Terra's dark aura) | `kh1_flowmotion_bbs_motion_fx(3, 408, 415, 1)` |
| Crown (Wingblade) | `kh1_flowmotion_bbs_motion_fx(1, 408, 418, 1)` |

### Ventus 408: groups 403, 404, 405, 406, 409, 414, 418, 419, 420
| Frame | File | Element | Bone | Group | Flags |
|---|---|---|---|---|---|
| 8 | p_st_03_spr | P_ST_SPR_ORA0_0 | mune | 403 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 403 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 403 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 403 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 403 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 403 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 403 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 403 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 403 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 403 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 403 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 403 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 403 | 0x10e |
| 8 | p_st_04_fib | P_ST_FIB_ORA0_0 | mune | 404 | 0x10e |
| 8 | p_st_04_fib | P_ST_FIB_ORA2_0 | mune | 404 | 0x6 |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 404 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 404 | 0x10e |
| 8 | p_st_05_dmd | P_ST_DMD_ORA0_0 | mune | 405 | 0x10e |
| 8 | p_st_05_dmd | P_ST_DMD_ORA2_0 | mune | 405 | 0x10a |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 405 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 405 | 0x10e |
| 8 | p_st_06_thd | P_ST_THD_ORA0_0 | mune | 406 | 0x10e |
| 8 | p_st_06_thd | P_ST_THD_ORA2_0 | mune | 406 | 0x6 |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 406 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 406 | 0x10e |
| 8 | p_st_09_cyc | P_ST_CYC_ORA0_0 | mune | 409 | 0x10e |
| 8 | p_st_09_cyc | P_ST_CYC_ORA2_0 | mune | 409 | 0x6 |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 409 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 409 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 409 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 409 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 409 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 409 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 409 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 409 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 409 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 409 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 409 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 409 | 0x10e |
| 8 | p_st_14_skr | P_ST_SKR_ORA0_0 | mune | 414 | 0x10e |
| 8 | p_st_14_skr | P_ST_SKR_ORA2_0 | mune | 414 | 0x6 |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 414 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 414 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 414 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 414 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 414 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 414 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 414 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 414 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 414 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 414 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 414 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 414 | 0x10e |
| 8 | p_st_18_wib | P_ST_WIB_ORA0_0 | mune | 418 | 0x10e |
| 8 | p_st_18_wib | P_ST_WIB_ORA2_0 | mune | 418 | 0x6 |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 418 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 418 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 418 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 418 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 418 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 418 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 418 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 418 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 418 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 418 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 418 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 418 | 0x10e |
| 8 | p_st_19_icb | P_ST_ICB_ORA0_0 | mune | 419 | 0x10e |
| 8 | p_st_19_icb | P_ST_ICB_ORA1_0 | mune | 419 | 0x6 |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 419 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 420 | 0x5 |
| 8 | p_st_20_rmr | P_ST_RMR_HIT0_0 | mune | 420 | 0x6 |
| 8 | p_st_20_rmr | P_ST_RMR_ORA0_0 | mune | 420 | 0x106 |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | mune | 420 | 0x10e |

### Aqua 408: groups 402, 404, 405, 406, 410, 413, 414, 419, 420
| Frame | File | Element | Bone | Group | Flags |
|---|---|---|---|---|---|
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 402 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 402 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 402 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 402 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 402 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 402 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 402 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 402 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 402 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 402 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 402 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 402 | 0x5 |
| 5.5 | p_st_02_mag | P_ST_MAG_ORA2_0 | mune | 402 | 0x4 |
| 5.5 | p_st_02_mag | P_ST_MAG_ORA0_0 | mune | 402 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 404 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 404 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 404 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 404 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 404 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 404 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 404 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 404 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 404 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 404 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 404 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 404 | 0x5 |
| 5.5 | p_st_04_fib | P_ST_FIB_ORA2_0 | mune | 404 | 0x6 |
| 5.5 | p_st_04_fib | P_ST_FIB_ORA0_0 | mune | 404 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 405 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 405 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 405 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 405 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 405 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 405 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 405 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 405 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 405 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 405 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 405 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 405 | 0x5 |
| 5.5 | p_st_05_dmd | P_ST_DMD_ORA2_0 | mune | 405 | 0x10a |
| 5.5 | p_st_05_dmd | P_ST_DMD_ORA0_0 | mune | 405 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 406 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 406 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 406 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 406 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 406 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 406 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 406 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 406 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 406 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 406 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 406 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 406 | 0x5 |
| 5.5 | p_st_06_thd | P_ST_THD_ORA2_0 | mune | 406 | 0x6 |
| 5.5 | p_st_06_thd | P_ST_THD_ORA0_0 | mune | 406 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 410 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 410 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 410 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 410 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 410 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 410 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 410 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 410 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 410 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 410 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 410 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 410 | 0x5 |
| 5.5 | p_st_10_blc | P_ST_BLC_ORA0_0 | mune | 410 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 413 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 413 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 413 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 413 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 413 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 413 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 413 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 413 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 413 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 413 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 413 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 413 | 0x5 |
| 5.5 | p_st_13_gsd | P_ST_GSD_ORA2_0 | mune | 413 | 0x106 |
| 5.5 | p_st_13_gsd | P_ST_GSD_ORA0_0 | mune | 413 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 414 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 414 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 414 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 414 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 414 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 414 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 414 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 414 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 414 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 414 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 414 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 414 | 0x5 |
| 5.5 | p_st_14_skr | P_ST_SKR_ORA2_0 | mune | 414 | 0x4 |
| 5.5 | p_st_14_skr | P_ST_SKR_ORA0_0 | mune | 414 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 419 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 419 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 419 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 419 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 419 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 419 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 419 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 419 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 419 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 419 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 419 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 419 | 0x5 |
| 5.5 | p_st_19_icb | P_ST_ICB_ORA1_0 | mune | 419 | 0x10a |
| 5.5 | p_st_19_icb | P_ST_ICB_ORA0_0 | mune | 419 | 0x10e |
| 5.5 | p_st_20_rmr | P_ST_RMR_ORA0_0 | mune | 420 | 0x106 |
| 5.5 | p_st_20_rmr | P_ST_RMR_HIT0_0 | mune | 420 | 0x4 |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 420 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 420 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 420 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 420 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 420 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA1_0 | mune | 420 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 420 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 420 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 420 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 420 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 420 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 420 | 0x10e |
| 5.5 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 420 | 0x5 |

### Terra 408: groups 401, 404, 405, 406, 410, 412, 415, 419, 420
| Frame | File | Element | Bone | Group | Flags |
|---|---|---|---|---|---|
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 401 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 401 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 401 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 401 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 401 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 401 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 401 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 401 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 401 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 401 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 401 | 0x10e |
| 8 | p_st_01_fam | P_ST_FAM_ORA0_0 | mune | 401 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 401 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 404 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 404 | 0x10e |
| 8 | p_st_04_fib | P_ST_FIB_ORA0_0 | mune | 404 | 0x10e |
| 8 | p_st_04_fib | P_ST_FIB_ORA2_0 | mune | 404 | 0x6 |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 404 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 405 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 405 | 0x10e |
| 8 | p_st_05_dmd | P_ST_DMD_ORA0_0 | mune | 405 | 0x10e |
| 8 | p_st_05_dmd | P_ST_DMD_ORA2_0 | mune | 405 | 0x108 |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 405 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 406 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 406 | 0x10e |
| 8 | p_st_06_thd | P_ST_THD_ORA0_0 | mune | 406 | 0x10e |
| 8 | p_st_06_thd | P_ST_THD_ORA2_0 | mune | 406 | 0x6 |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 406 | 0x5 |
| 8 | p_st_10_blc | P_ST_BLC_ORA0_0 | mune | 410 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 410 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 410 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 410 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 410 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 410 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 410 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 410 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 410 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 410 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 410 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 410 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 410 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 412 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 412 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 412 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 412 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 412 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 412 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 412 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 412 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 412 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 412 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 412 | 0x10e |
| 8 | p_st_12_rob | P_ST_ROB_ORA0_0 | mune | 412 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 412 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA3_0 | R_momo | 415 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA3_0 | L_momo | 415 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA3_0 | R_ude2 | 415 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA3_0 | R_ude1 | 415 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA3_0 | L_ude1 | 415 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA4_0 | koshi | 415 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA4_0 | R_te | 415 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA4_0 | atama | 415 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA4_0 | atama | 415 | 0x10e |
| 8 | p_st_15_dim | P_ST_DIM_ORA0_0 | mune | 415 | 0x10e |
| 8 | p_st_15_dim | P_ST_DIM_ORA2_0 | mune | 415 | 0x6 |
| 8 | p_st_15_dim | P_ST_DIM_ORA3_0 | L_ude2 | 415 | 0x10e |
| 8 | p_st_15_dim | P_ST_DIM_ORA4_0 | L_te | 415 | 0x106 |
| 8 | p_st_15_dim | P_ST_DIM_ORA5_0 | mune | 415 | 0x106 |
| 8 | p_st_15_dim | P_ST_DIM_ORA5_0 | L_te | 415 | 0x106 |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 415 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 419 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 419 | 0x10e |
| 8 | p_st_19_icb | P_ST_ICB_ORA0_0 | mune | 419 | 0x10e |
| 8 | p_st_19_icb | P_ST_ICB_ORA1_0 | mune | 419 | 0x108 |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 419 | 0x5 |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | mune | 420 | 0x10e |
| 8 | p_st_20_rmr | P_ST_RMR_HIT0_0 | mune | 420 | 0x5 |
| 8 | p_st_20_rmr | P_ST_RMR_ORA0_0 | mune | 420 | 0x106 |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | atama | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | L_te | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | R_te | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA1_0 | koshi | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude1 | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_ude2 | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude1 | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_ude2 | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | L_momo | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_ORA0_0 | R_momo | 420 | 0x10e |
| 8 | p_st_com_00 | P_ST_COM_CAL1_0 | mune | 420 | 0x5 |

