export const asyncFunc = async function() {
      console.log("Mock: ⏳ リクエストを受信。3秒間の処理を開始します...");
      const start = performance.now();

      // --- ここが核心部分 ---
      // Promise と setTimeout を使って、
      // ブラウザをフリーズさせずに(非同期で) 3000ms 待機します
      await new Promise(resolve => setTimeout(resolve, 3000));
      // ---------------------

      const end = performance.now();
      const elapsed = ((end - start) / 1000).toFixed(2);
      
      console.log(`Mock: ✅ 完了しました (経過時間: ${elapsed}秒)`);
      
      // Rust側に返す文字列
      return JSON.stringify({
        status: "success",
        message: "Heavy task finished",
        duration: `${elapsed}s`
      });
    };
