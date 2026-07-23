// Data models mirroring the LibrePay API JSON shapes.

class Transaction {
  final String id;
  final String reference;
  final int amount;
  final String currency;
  final String provider;
  final String phone;
  final String status; // pending | success | failed
  final String? providerRef;
  final DateTime createdAt;
  final DateTime updatedAt;

  Transaction({
    required this.id,
    required this.reference,
    required this.amount,
    required this.currency,
    required this.provider,
    required this.phone,
    required this.status,
    required this.providerRef,
    required this.createdAt,
    required this.updatedAt,
  });

  factory Transaction.fromJson(Map<String, dynamic> j) => Transaction(
        id: j['id'] as String,
        reference: j['reference'] as String,
        amount: j['amount'] as int,
        currency: j['currency'] as String,
        provider: j['provider'] as String,
        phone: j['phone'] as String,
        status: j['status'] as String,
        providerRef: j['provider_ref'] as String?,
        createdAt: DateTime.parse(j['created_at'] as String),
        updatedAt: DateTime.parse(j['updated_at'] as String),
      );
}

class ProviderStat {
  final String provider;
  final int count;
  final int successVolume;

  ProviderStat({
    required this.provider,
    required this.count,
    required this.successVolume,
  });

  factory ProviderStat.fromJson(Map<String, dynamic> j) => ProviderStat(
        provider: j['provider'] as String,
        count: j['count'] as int,
        successVolume: j['success_volume'] as int,
      );
}

class Stats {
  final int total;
  final int pending;
  final int success;
  final int failed;
  final double successRate;
  final int totalVolume;
  final List<ProviderStat> byProvider;

  Stats({
    required this.total,
    required this.pending,
    required this.success,
    required this.failed,
    required this.successRate,
    required this.totalVolume,
    required this.byProvider,
  });

  factory Stats.fromJson(Map<String, dynamic> j) => Stats(
        total: j['total'] as int,
        pending: j['pending'] as int,
        success: j['success'] as int,
        failed: j['failed'] as int,
        successRate: (j['success_rate'] as num).toDouble(),
        totalVolume: j['total_volume'] as int,
        byProvider: (j['by_provider'] as List)
            .map((e) => ProviderStat.fromJson(e as Map<String, dynamic>))
            .toList(),
      );
}
