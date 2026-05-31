//! K-means clustering for user usage patterns.

/// K-means clustering for user usage patterns
pub struct UsageClusterer;

impl UsageClusterer {
    /// Cluster users by usage patterns using k-means algorithm.
    ///
    /// Iterates centroid assignment until convergence or max 100 iterations.
    /// Falls back to single-cluster assignment when data is insufficient.
    pub fn cluster_users(
        usage_data: &[(String, Vec<f64>)],
        n_clusters: usize,
    ) -> Vec<(String, usize)> {
        if usage_data.is_empty() || n_clusters == 0 {
            return Vec::new();
        }
        let k = n_clusters.min(usage_data.len());
        let dim = usage_data.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
        if dim == 0 || k <= 1 {
            return usage_data.iter().map(|(id, _)| (id.clone(), 0)).collect();
        }

        // Pad feature vectors to uniform dimension
        let points: Vec<Vec<f64>> = usage_data
            .iter()
            .map(|(_, v)| {
                let mut p = v.clone();
                p.resize(dim, 0.0);
                p
            })
            .collect();

        // Initialize centroids from first k distinct points
        let mut centroids: Vec<Vec<f64>> = points.iter().take(k).cloned().collect();
        let mut assignments = vec![0usize; points.len()];
        let max_iter = 100;

        for _ in 0..max_iter {
            // Assignment step: assign each point to nearest centroid
            let mut changed = false;
            for (i, point) in points.iter().enumerate() {
                let nearest = centroids
                    .iter()
                    .enumerate()
                    .map(|(ci, c)| {
                        let dist: f64 = point
                            .iter()
                            .zip(c.iter())
                            .map(|(a, b)| (a - b) * (a - b))
                            .sum();
                        (ci, dist)
                    })
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(ci, _)| ci)
                    .unwrap_or(0);
                if assignments[i] != nearest {
                    assignments[i] = nearest;
                    changed = true;
                }
            }

            if !changed {
                break;
            }

            // Update step: recompute centroids as mean of assigned points
            let mut new_centroids = vec![vec![0.0; dim]; k];
            let mut counts = vec![0usize; k];
            for (i, point) in points.iter().enumerate() {
                let c = assignments[i];
                counts[c] += 1;
                for (j, val) in point.iter().enumerate() {
                    new_centroids[c][j] += val;
                }
            }
            for (ci, centroid) in new_centroids.iter_mut().enumerate() {
                if counts[ci] > 0 {
                    for val in centroid.iter_mut() {
                        *val /= counts[ci] as f64;
                    }
                } else {
                    // Keep old centroid for empty clusters
                    *centroid = centroids[ci].clone();
                }
            }
            centroids = new_centroids;
        }

        usage_data
            .iter()
            .enumerate()
            .map(|(i, (id, _))| (id.clone(), assignments[i]))
            .collect()
    }
}
