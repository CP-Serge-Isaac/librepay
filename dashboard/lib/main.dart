import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import 'api.dart';
import 'models.dart';
import 'theme.dart';

void main() => runApp(const LibrePayDashboard());

class LibrePayDashboard extends StatelessWidget {
  const LibrePayDashboard({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'LibrePay — Dashboard',
      debugShowCheckedModeBanner: false,
      theme: buildTheme(),
      home: const DashboardPage(),
    );
  }
}

class DashboardPage extends StatefulWidget {
  const DashboardPage({super.key});

  @override
  State<DashboardPage> createState() => _DashboardPageState();
}

class _DashboardPageState extends State<DashboardPage> {
  final _api = ApiClient();
  final _money = NumberFormat.decimalPattern('fr');
  final _dt = DateFormat('dd/MM HH:mm:ss');

  bool _loading = true;
  bool? _online;
  String? _error;
  Stats? _stats;
  List<Transaction> _txns = [];
  String _statusFilter = 'all';

  @override
  void initState() {
    super.initState();
    _refresh();
  }

  Future<void> _refresh() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final online = await _api.health();
      final status = _statusFilter == 'all' ? null : _statusFilter;
      final results = await Future.wait([
        _api.stats(),
        _api.payments(status: status, limit: 200),
      ]);
      setState(() {
        _online = online;
        _stats = results[0] as Stats;
        _txns = results[1] as List<Transaction>;
        _loading = false;
      });
    } catch (e) {
      setState(() {
        _online = false;
        _error = '$e';
        _loading = false;
      });
    }
  }

  Future<void> _seedDemo() async {
    try {
      final n = _txns.length;
      for (var i = 1; i <= 3; i++) {
        await _api.createPayment(
          amount: (n + i) * 1000,
          provider: 'mock',
          phone: '+2267${(1000000 + n + i)}',
          reference: 'DEMO-${n + i}',
        );
      }
      await _refresh();
    } catch (e) {
      _showError('$e');
    }
  }

  void _showError(String m) {
    if (!mounted) return;
    ScaffoldMessenger.of(context)
        .showSnackBar(SnackBar(content: Text(m), backgroundColor: Brand.failed));
  }

  Future<void> _editBaseUrl() async {
    final ctrl = TextEditingController(text: _api.baseUrl);
    final res = await showDialog<String>(
      context: context,
      builder: (c) => AlertDialog(
        title: const Text('API base URL'),
        content: TextField(
          controller: ctrl,
          decoration: const InputDecoration(hintText: 'http://localhost:8080'),
        ),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(c), child: const Text('Annuler')),
          FilledButton(
              onPressed: () => Navigator.pop(c, ctrl.text.trim()),
              child: const Text('OK')),
        ],
      ),
    );
    if (res != null && res.isNotEmpty) {
      setState(() => _api.baseUrl = res);
      _refresh();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            _TopBar(
              online: _online,
              baseUrl: _api.baseUrl,
              loading: _loading,
              onRefresh: _refresh,
              onEditUrl: _editBaseUrl,
              onSeed: _seedDemo,
            ),
            Expanded(
              child: _error != null
                  ? _ErrorView(error: _error!, onRetry: _refresh)
                  : RefreshIndicator(
                      onRefresh: _refresh,
                      child: ListView(
                        padding: const EdgeInsets.all(24),
                        children: [
                          _kpiRow(),
                          const SizedBox(height: 24),
                          _providerBreakdown(),
                          const SizedBox(height: 24),
                          _transactionsSection(),
                        ],
                      ),
                    ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _kpiRow() {
    final s = _stats;
    final rate = s == null ? '—' : '${(s.successRate * 100).toStringAsFixed(1)}%';
    return LayoutBuilder(builder: (context, c) {
      final wide = c.maxWidth > 900;
      final cards = [
        _KpiCard(
            label: 'Transactions',
            value: s?.total.toString() ?? '—',
            icon: Icons.receipt_long,
            color: Brand.ink),
        _KpiCard(
            label: 'Volume (succès)',
            value: s == null ? '—' : '${_money.format(s.totalVolume)} XOF',
            icon: Icons.payments,
            color: Brand.emerald),
        _KpiCard(
            label: 'Taux de succès',
            value: rate,
            icon: Icons.trending_up,
            color: Brand.emeraldDark),
        _KpiCard(
            label: 'En attente',
            value: s?.pending.toString() ?? '—',
            icon: Icons.hourglass_top,
            color: Brand.pending),
      ];
      return Wrap(
        spacing: 16,
        runSpacing: 16,
        children: cards
            .map((w) => SizedBox(
                width: wide ? (c.maxWidth - 48) / 4 : (c.maxWidth - 16) / 2,
                child: w))
            .toList(),
      );
    });
  }

  Widget _providerBreakdown() {
    final s = _stats;
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const _SectionTitle('Par opérateur'),
            const SizedBox(height: 12),
            if (s == null || s.byProvider.isEmpty)
              const Text('Aucune donnée', style: TextStyle(color: Brand.slate))
            else
              ...s.byProvider.map((p) => Padding(
                    padding: const EdgeInsets.symmetric(vertical: 6),
                    child: Row(
                      children: [
                        const CircleAvatar(
                          radius: 4,
                          backgroundColor: Brand.emerald,
                        ),
                        const SizedBox(width: 10),
                        Expanded(
                            child: Text(p.provider,
                                style: const TextStyle(
                                    fontWeight: FontWeight.w600))),
                        Text('${p.count} tx',
                            style: const TextStyle(color: Brand.slate)),
                        const SizedBox(width: 20),
                        Text('${_money.format(p.successVolume)} XOF',
                            style: const TextStyle(
                                fontWeight: FontWeight.w600,
                                color: Brand.emeraldDark)),
                      ],
                    ),
                  )),
          ],
        ),
      ),
    );
  }

  Widget _transactionsSection() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const _SectionTitle('Transactions'),
                const Spacer(),
                _StatusFilter(
                  value: _statusFilter,
                  onChanged: (v) {
                    setState(() => _statusFilter = v);
                    _refresh();
                  },
                ),
              ],
            ),
            const SizedBox(height: 8),
            if (_txns.isEmpty)
              const Padding(
                padding: EdgeInsets.symmetric(vertical: 32),
                child: Center(
                    child: Text('Aucune transaction',
                        style: TextStyle(color: Brand.slate))),
              )
            else
              SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: DataTable(
                  headingTextStyle: const TextStyle(
                      fontWeight: FontWeight.w700, color: Brand.ink),
                  columns: const [
                    DataColumn(label: Text('Référence')),
                    DataColumn(label: Text('Opérateur')),
                    DataColumn(label: Text('Téléphone')),
                    DataColumn(label: Text('Montant')),
                    DataColumn(label: Text('Statut')),
                    DataColumn(label: Text('Créé')),
                  ],
                  rows: _txns
                      .map((t) => DataRow(cells: [
                            DataCell(Text(t.reference,
                                style: const TextStyle(
                                    fontWeight: FontWeight.w600))),
                            DataCell(Text(t.provider)),
                            DataCell(Text(t.phone)),
                            DataCell(Text('${_money.format(t.amount)} ${t.currency}')),
                            DataCell(_StatusChip(t.status)),
                            DataCell(Text(_dt.format(t.createdAt.toLocal()),
                                style: const TextStyle(color: Brand.slate))),
                          ]))
                      .toList(),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

// ─── Widgets ────────────────────────────────────────────────────────────────

class _TopBar extends StatelessWidget {
  final bool? online;
  final String baseUrl;
  final bool loading;
  final VoidCallback onRefresh;
  final VoidCallback onEditUrl;
  final VoidCallback onSeed;

  const _TopBar({
    required this.online,
    required this.baseUrl,
    required this.loading,
    required this.onRefresh,
    required this.onEditUrl,
    required this.onSeed,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
      decoration: const BoxDecoration(
        color: Colors.white,
        border: Border(bottom: BorderSide(color: Brand.border)),
      ),
      child: Row(
        children: [
          Container(
            width: 34,
            height: 34,
            decoration: BoxDecoration(
              color: Brand.emerald,
              borderRadius: BorderRadius.circular(9),
            ),
            child: const Icon(Icons.hub, color: Colors.white, size: 20),
          ),
          const SizedBox(width: 12),
          const Text('LibrePay',
              style: TextStyle(
                  fontSize: 20, fontWeight: FontWeight.w800, color: Brand.ink)),
          const SizedBox(width: 8),
          const Padding(
            padding: EdgeInsets.only(top: 3),
            child: Text('Dashboard',
                style: TextStyle(color: Brand.slate, fontSize: 14)),
          ),
          const Spacer(),
          _StatusDot(online: online),
          const SizedBox(width: 12),
          OutlinedButton.icon(
            onPressed: onEditUrl,
            icon: const Icon(Icons.link, size: 16),
            label: Text(baseUrl, style: const TextStyle(fontSize: 12)),
          ),
          const SizedBox(width: 8),
          OutlinedButton.icon(
            onPressed: onSeed,
            icon: const Icon(Icons.add, size: 16),
            label: const Text('Démo'),
          ),
          const SizedBox(width: 8),
          FilledButton.icon(
            onPressed: loading ? null : onRefresh,
            icon: loading
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(
                        strokeWidth: 2, color: Colors.white))
                : const Icon(Icons.refresh, size: 18),
            label: const Text('Actualiser'),
          ),
        ],
      ),
    );
  }
}

class _StatusDot extends StatelessWidget {
  final bool? online;
  const _StatusDot({required this.online});

  @override
  Widget build(BuildContext context) {
    final c = online == null
        ? Brand.slate
        : (online! ? Brand.success : Brand.failed);
    final label = online == null
        ? '…'
        : (online! ? 'API en ligne' : 'API hors ligne');
    return Row(children: [
      Container(
          width: 9,
          height: 9,
          decoration: BoxDecoration(color: c, shape: BoxShape.circle)),
      const SizedBox(width: 6),
      Text(label, style: TextStyle(color: c, fontSize: 12)),
    ]);
  }
}

class _KpiCard extends StatelessWidget {
  final String label;
  final String value;
  final IconData icon;
  final Color color;
  const _KpiCard(
      {required this.label,
      required this.value,
      required this.icon,
      required this.color});

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  padding: const EdgeInsets.all(8),
                  decoration: BoxDecoration(
                    color: color.withValues(alpha: 0.10),
                    borderRadius: BorderRadius.circular(9),
                  ),
                  child: Icon(icon, color: color, size: 18),
                ),
              ],
            ),
            const SizedBox(height: 14),
            Text(value,
                style: const TextStyle(
                    fontSize: 24, fontWeight: FontWeight.w800, color: Brand.ink)),
            const SizedBox(height: 4),
            Text(label, style: const TextStyle(color: Brand.slate, fontSize: 13)),
          ],
        ),
      ),
    );
  }
}

class _SectionTitle extends StatelessWidget {
  final String text;
  const _SectionTitle(this.text);
  @override
  Widget build(BuildContext context) => Text(text,
      style: const TextStyle(
          fontSize: 16, fontWeight: FontWeight.w700, color: Brand.ink));
}

class _StatusChip extends StatelessWidget {
  final String status;
  const _StatusChip(this.status);
  @override
  Widget build(BuildContext context) {
    final c = Brand.statusColor(status);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: c.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(20),
      ),
      child: Text(status,
          style:
              TextStyle(color: c, fontWeight: FontWeight.w700, fontSize: 12)),
    );
  }
}

class _StatusFilter extends StatelessWidget {
  final String value;
  final ValueChanged<String> onChanged;
  const _StatusFilter({required this.value, required this.onChanged});

  @override
  Widget build(BuildContext context) {
    return DropdownButton<String>(
      value: value,
      underline: const SizedBox.shrink(),
      borderRadius: BorderRadius.circular(12),
      items: const [
        DropdownMenuItem(value: 'all', child: Text('Tous')),
        DropdownMenuItem(value: 'pending', child: Text('En attente')),
        DropdownMenuItem(value: 'success', child: Text('Succès')),
        DropdownMenuItem(value: 'failed', child: Text('Échoué')),
      ],
      onChanged: (v) => onChanged(v ?? 'all'),
    );
  }
}

class _ErrorView extends StatelessWidget {
  final String error;
  final VoidCallback onRetry;
  const _ErrorView({required this.error, required this.onRetry});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.cloud_off, size: 48, color: Brand.slate),
          const SizedBox(height: 16),
          const Text('Impossible de joindre l’API LibrePay',
              style: TextStyle(fontWeight: FontWeight.w700, color: Brand.ink)),
          const SizedBox(height: 8),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 40),
            child: Text(error,
                textAlign: TextAlign.center,
                style: const TextStyle(color: Brand.slate, fontSize: 12)),
          ),
          const SizedBox(height: 16),
          FilledButton(onPressed: onRetry, child: const Text('Réessayer')),
        ],
      ),
    );
  }
}
