<script>
  import { onMount } from 'svelte';
  import Chart from 'chart.js/auto';

  export let benchmarkResults = [];

  let chartCanvas;
  let chart;

  onMount(() => {
    if (benchmarkResults.length > 0) {
      createChart();
    }
  });

  $: if (benchmarkResults.length > 0 && chartCanvas) {
    updateChart();
  }

  function createChart() {
    const ctx = chartCanvas.getContext('2d');

    // Calculate average performance metrics for each language
    const languageData = benchmarkResults.map(result => {
      if (!result.benchmarks || result.benchmarks.length === 0) {
        return {
          language: result.language,
          avgTime: 0,
          avgOps: 0,
          benchmarkCount: 0
        };
      }

      const avgTime = result.benchmarks.reduce((sum, b) => sum + (b.avg_time_ms || 0), 0) / result.benchmarks.length;
      const avgOps = result.benchmarks.reduce((sum, b) => sum + (b.ops_per_second || 0), 0) / result.benchmarks.length;

      return {
        language: result.language,
        avgTime: avgTime,
        avgOps: avgOps,
        benchmarkCount: result.benchmarks.length
      };
    });


    const data = {
      labels: languageData.map(d => d.language),
      datasets: [
        {
          label: 'Average Execution Time (ms)',
          data: languageData.map(d => d.avgTime),
          backgroundColor: 'rgba(255, 99, 132, 0.6)',
          borderColor: 'rgba(255, 99, 132, 1)',
          borderWidth: 2,
          yAxisID: 'y'
        },
        {
          label: 'Average Operations/Second',
          data: languageData.map(d => d.avgOps),
          backgroundColor: 'rgba(54, 162, 235, 0.6)',
          borderColor: 'rgba(54, 162, 235, 1)',
          borderWidth: 2,
          yAxisID: 'y1'
        }
      ]
    };

    const config = {
      type: 'bar',
      data: data,
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: {
          mode: 'index',
          intersect: false,
        },
        plugins: {
          title: {
            display: true,
            text: 'Benchmark Performance by Language Implementation'
          },
          legend: {
            position: 'top',
          },
          tooltip: {
            callbacks: {
              afterLabel: function(context) {
                const langData = languageData[context.dataIndex];
                return `Benchmarks: ${langData.benchmarkCount}`;
              }
            }
          }
        },
        scales: {
          x: {
            title: {
              display: true,
              text: 'Language Implementation'
            }
          },
          y: {
            type: 'linear',
            display: true,
            position: 'left',
            title: {
              display: true,
              text: 'Average Execution Time (ms)'
            },
            beginAtZero: true
          },
          y1: {
            type: 'linear',
            display: true,
            position: 'right',
            title: {
              display: true,
              text: 'Average Operations/Second'
            },
            beginAtZero: true,
            grid: {
              drawOnChartArea: false,
            },
          }
        }
      }
    };

    chart = new Chart(ctx, config);
  }

  function updateChart() {
    if (chart) {
      chart.destroy();
      createChart();
    }
  }
</script>

<div class="chart-container">
  <canvas bind:this={chartCanvas}></canvas>
</div>

<style>
  .chart-container {
    position: relative;
    height: 450px;
    width: 100%;
    margin: 30px 0;
    padding: 20px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.05);
  }

  /* Responsive spacing */
  @media (max-width: 768px) {
    .chart-container {
      height: 350px;
      margin: 20px 0;
      padding: 15px;
    }
  }

  @media (max-width: 480px) {
    .chart-container {
      height: 300px;
      margin: 15px 0;
      padding: 10px;
    }
  }
</style>
