// Thin HTTP client for the LibrePay API.

import 'dart:convert';
import 'package:http/http.dart' as http;

import 'models.dart';

class ApiClient {
  /// Base URL of the LibrePay API. Configurable from the UI.
  String baseUrl;

  ApiClient({this.baseUrl = 'http://localhost:8080'});

  Uri _u(String path, [Map<String, String>? q]) =>
      Uri.parse('$baseUrl$path').replace(queryParameters: q);

  Future<bool> health() async {
    try {
      final r = await http.get(_u('/health'));
      return r.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  Future<Stats> stats() async {
    final r = await http.get(_u('/v1/stats'));
    if (r.statusCode != 200) {
      throw Exception('stats failed: ${r.statusCode}');
    }
    return Stats.fromJson(jsonDecode(r.body) as Map<String, dynamic>);
  }

  Future<List<Transaction>> payments({
    String? status,
    String? provider,
    int limit = 100,
  }) async {
    final q = <String, String>{'limit': '$limit'};
    if (status != null) q['status'] = status;
    if (provider != null) q['provider'] = provider;
    final r = await http.get(_u('/v1/payments', q));
    if (r.statusCode != 200) {
      throw Exception('payments failed: ${r.statusCode}');
    }
    final list = jsonDecode(r.body) as List;
    return list
        .map((e) => Transaction.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  /// Create a payment — handy for generating demo data from the dashboard.
  Future<Transaction> createPayment({
    required int amount,
    required String provider,
    required String phone,
    required String reference,
  }) async {
    final r = await http.post(
      _u('/v1/payments'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'amount': amount,
        'provider': provider,
        'phone': phone,
        'reference': reference,
      }),
    );
    if (r.statusCode != 200) {
      throw Exception('create failed: ${r.statusCode} ${r.body}');
    }
    return Transaction.fromJson(jsonDecode(r.body) as Map<String, dynamic>);
  }
}
