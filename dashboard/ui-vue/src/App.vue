<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand-wrap">
        <div class="brand-icon">K</div>
        <div>
          <h1>KATARA</h1>
          <p class="brand-sub">Sovereign AI Context OS</p>
        </div>
      </div>
      <nav>
        <RouterLink :to="{ name: 'overview' }"><span class="nav-icon">📊</span> Overview</RouterLink>
        <RouterLink :to="{ name: 'flow' }"><span class="nav-icon">🔀</span> AI Flow</RouterLink>
        <RouterLink :to="{ name: 'memory' }"><span class="nav-icon">🧠</span> Memory</RouterLink>
        <RouterLink :to="{ name: 'insights' }"><span class="nav-icon">💡</span> Insights</RouterLink>
        <RouterLink :to="{ name: 'benchmarks' }"><span class="nav-icon">📈</span> Benchmarks</RouterLink>
      </nav>
      <div class="sidebar-footer">
        <span class="version-tag">v7.0.0</span>
      </div>
    </aside>
    <main class="content">
      <RouterView />
    </main>
  </div>
</template>
