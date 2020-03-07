#include <bits/stdc++.h>
// #pragma GCC optimize("Ofast")
// #pragma GCC target("avx,avx2,fma")
// #pragma GCC optimize("unroll-loops")
using namespace std;
typedef long long ll;

namespace Math {
    ll modpow(ll b, ll p, ll m) {
        ll r = 1;
        for (; p; p >>= 1, b = b * b % m)
            if (p & 1)
                r = r * b % m;
        return r;
    }

    ll modinv(ll b, ll m) { return modpow(b, m - 2, m); }

    // Least significant bit.
    ll lsb(ll x) { return x & (-x); }

    template <class T> const T inf() { return 0; }
    template <> const ll inf<ll>() { return 0x3f3f3f3f3f3f3f3fll; }
    template <> const int inf<int>() { return 0x3f3f3f3f; }

    template <class T> T gcd(T a, T b) { return abs(__gcd(a, b)); }
    template <class T> T clamp(T n, T l, T r) { return n < l ? l : n > r ? r : n; }
    template <class T> T lcm(T a, T b) { return abs(a / gcd(a, b) * b); }
    template <class T> T sgn(T n) { return n == 0 ? 0 : n > 0 ? 1 : -1; }

    // Generic modular integer.
    template<int MOD, typename IntType = int>
    struct GModint {
        IntType val;

        GModint() : val(0) {}

        GModint(int v, bool safe = false) { val = safe ? v : ((v % MOD) + MOD) % MOD; }

        GModint(ll v, bool safe = false) { val = safe ? v : ((v % MOD) + MOD) % MOD; }

        explicit operator bool() const { return val != 0; }

        const bool operator==(const GModint &g) const { return val == g.val; }

        const bool operator!=(const GModint &g) const { return val != g.val; }

        const GModint operator+=(const GModint &g) { return *this = *this + g; }

        const GModint operator-=(const GModint &g) { return *this = *this - g; }

        const GModint operator*=(const GModint &g) { return *this = *this * g; }

        const GModint operator/=(const GModint &g) { return *this = *this / g; }

        const GModint operator^=(const GModint &g) { return *this = *this ^ g; }

        const GModint operator-() const { return GModint(val == 0 ? 0 : MOD - val, true); }

        const GModint operator+(const GModint &g) const {
            ll newval = val + g.val;
            if (newval >= MOD)
                newval -= MOD;
            return GModint(newval, true);
        }

        const GModint operator-(const GModint &g) const { return *this + (-g); }

        const GModint operator*(const GModint &g) const { return GModint(1ll * val * g.val); }

        const GModint operator/(const GModint &g) const { return *this * g.inv(); }

        template<class T>
        const GModint operator^(const T t) const { return GModint(modpow(val, t, MOD)); }

        const GModint inv() const { return GModint(modinv(val, MOD)); }
    };

    using Modint = GModint<1000000007>;
}

vector<int> FR[100005], BK[100005];
int dsu[100005], dsu_size[100005];
bool vis[100005];

int dfind(int u) {
    if (dsu[u] == u) return u;
    return dsu[u] = dfind(dsu[u]);
}

void dmerge(int u, int v) {
    u = dfind(u), v = dfind(v);
    if (u == v) return;
    dsu_size[u] += dsu_size[v];
    dsu[v] = u;
}

void dfs(int x) {
    if (vis[x]) return;
    vis[x] = true;
    for (int h : FR[x]) {
        dmerge(h, x);
        dfs(h);
    }
}

int main() {
    ios_base::sync_with_stdio(false);
    cin.tie(0);

    int n, m, k; cin >> n >> m >> k;
    for (int i = 1; i <= n; i++) dsu[i] = i, dsu_size[i] = 1;
    for (int i = 1; i <= m; i++) {
        int a, b; cin >> a >> b;
        FR[a].push_back(b); FR[b].push_back(a);
    }
    for (int i = 1; i <= k; i++) {
        int a, b; cin >> a >> b;
        BK[a].push_back(b); BK[b].push_back(a);
    }
    for (int i = 1; i <= n; i++) dfs(i);
    for (int i = 1; i <= n; i++) {
        int ans = dsu_size[dfind(i)];
        ans--; // self
        for (int h : FR[i]) if (dfind(h) == dfind(i)) ans--;
        for (int h : BK[i]) if (dfind(h) == dfind(i)) ans--;
        cout << ans << " ";
    }
}
