import 'package:flutter/material.dart';

/// LibrePay brand palette — emerald (money/open) on cool slate neutrals.
class Brand {
  static const emerald = Color(0xFF0EA372);
  static const emeraldDark = Color(0xFF0B7E58);
  static const ink = Color(0xFF0F1B2D);
  static const slate = Color(0xFF64748B);
  static const surface = Color(0xFFF6F8FA);
  static const border = Color(0xFFE2E8F0);

  static const success = Color(0xFF0EA372);
  static const pending = Color(0xFFD97706);
  static const failed = Color(0xFFDC2626);

  static Color statusColor(String s) => switch (s) {
        'success' => success,
        'pending' => pending,
        'failed' => failed,
        _ => slate,
      };
}

ThemeData buildTheme() {
  final base = ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.fromSeed(
      seedColor: Brand.emerald,
      primary: Brand.emerald,
    ),
    scaffoldBackgroundColor: Brand.surface,
    fontFamily: 'Inter',
  );
  return base.copyWith(
    cardTheme: CardThemeData(
      elevation: 0,
      color: Colors.white,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
        side: const BorderSide(color: Brand.border),
      ),
    ),
    dividerColor: Brand.border,
  );
}
