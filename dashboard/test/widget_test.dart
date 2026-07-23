import 'package:flutter_test/flutter_test.dart';

import 'package:librepay_dashboard/main.dart';

void main() {
  testWidgets('Dashboard renders the LibrePay brand', (tester) async {
    await tester.pumpWidget(const LibrePayDashboard());
    expect(find.text('LibrePay'), findsOneWidget);
  });
}
