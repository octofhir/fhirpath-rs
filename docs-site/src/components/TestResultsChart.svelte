<script>
  import { onMount } from 'svelte';
  import Chart from 'chart.js/auto';

  export let testResults = [];

  let chartCanvas;
  let chart;

  onMount(() => {
    if (testResults.length > 0) {
      createChart();
    }
  });

  $: if (testResults.length > 0 && chartCanvas) {
    updateChart();
  }

  function createChart() {
    const ctx = chartCanvas.getContext('2d');

    const data = {
      labels: testResults.map(result => result.language),
      datasets: [
        {
          label: 'Tests Passed',
          data: testResults.map(result => result.summary?.passed || 0),
          backgroundColor: 'rgba(75, 192, 192, 0.6)',
          borderColor: 'rgba(75, 192, 192, 1)',
          borderWidth: 2
        },
        {
          label: 'Tests Failed',
          data: testResults.map(result => result.summary?.failed || 0),
          backgroundColor: 'rgba(255, 99, 132, 0.6)',
          borderColor: 'rgba(255, 99, 132, 1)',
          borderWidth: 2
        },
        {
          label: 'Tests with Errors',
          data: testResults.map(result => result.summary?.errors || 0),
          backgroundColor: 'rgba(255, 206, 86, 0.6)',
          borderColor: 'rgba(255, 206, 86, 1)',
          borderWidth: 2
        }
      ]
    };

    const config = {
      type: 'bar',
      data: data,
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          title: {
            display: true,
            text: 'Test Results by Language Implementation'
          },
          legend: {
            position: 'top',
          }
        },
        scales: {
          y: {
            beginAtZero: true,
            title: {
              display: true,
              text: 'Number of Tests'
            }
          },
          x: {
            title: {
              display: true,
              text: 'Language Implementation'
            }
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
