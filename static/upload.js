document.addEventListener('DOMContentLoaded', function(){
    // Store items in memory
    let items = [];

    // HS Code search function
    document.getElementById('search-hs').addEventListener('click', async function() {
        const hint = document.getElementById('hs-hint').value.trim();
        if (!hint) {
            alert('Please enter a search term');
            return;
        }
        
        const resultsCard = document.getElementById('hs-code-results');
        resultsCard.classList.remove('d-none');
        document.getElementById('hs-results-container').innerHTML = '<p>Searching...</p>';
        
        try {
            const response = await fetch('/search-hs-codes', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ hint })
            });
            
            if (!response.ok) throw new Error('Search failed');
            
            const data = await response.json();
            
            if (data.results.length === 0) {
                document.getElementById('hs-results-container').innerHTML = 
                    '<p>No results found. Try a different search term.</p>';
                return;
            }
            
            let html = '<div class="list-group">';
            data.results.forEach(result => {
                html += `
                    <a href="#" class="list-group-item list-group-item-action hs-code-item" 
                    data-code="${result.code}">
                        <strong>${result.code}</strong>: ${result.description}
                    </a>
                `;
            });
            html += '</div>';
            
            document.getElementById('hs-results-container').innerHTML = html;
            
            // Add click handlers to HS code items
            document.querySelectorAll('.hs-code-item').forEach(item => {
                item.addEventListener('click', function(e) {
                    e.preventDefault();
                    document.getElementById('hs-code').value = this.dataset.code;
                });
            });
        } catch (error) {
            document.getElementById('hs-results-container').innerHTML = 
                `<p class="text-danger">Error: ${error.message}</p>`;
        }
    });

    // Add item to list
    document.getElementById('item-form').addEventListener('submit', function(e) {
        e.preventDefault();
        
        const item = {
            description: document.getElementById('item-desc').value,
            packages: document.getElementById('packages').value,
            cost: document.getElementById('cost').value,
            units: document.getElementById('units').value,
            weight: document.getElementById('weight').value,
            hs_code: document.getElementById('hs-code').value
        };
        
        items.push(item);
        updateItemsList();
        
        // Reset form
        this.reset();
        document.getElementById('hs-code-results').classList.add('d-none');
    });

    // Update items list table
    function updateItemsList() {
        const tbody = document.getElementById('items-list');
        tbody.innerHTML = '';
        
        items.forEach((item, index) => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${item.description}</td>
                <td>${item.packages}</td>
                <td>${item.cost}</td>
                <td>${item.units}</td>
                <td>${item.weight}</td>
                <td>${item.hs_code}</td>
                <td>
                    <button class="btn btn-sm btn-danger remove-item" data-index="${index}">
                        Remove
                    </button>
                </td>
            `;
            tbody.appendChild(row);
        });
        
        // Add event listeners to remove buttons
        document.querySelectorAll('.remove-item').forEach(button => {
            button.addEventListener('click', function() {
                const index = parseInt(this.dataset.index);
                items.splice(index, 1);
                updateItemsList();
            });
        });
    }

    // Process to PDF
    document.getElementById('process-btn').addEventListener('click', async function() {
        if (items.length === 0) {
            alert('Please add at least one item');
            return;
        }
        
        document.getElementById('loading').style.display = 'block';
        
        try {
            //transform data
            /*const payload = items.map(item => ({
                description: item.description,
                packages: item.packages,
                cost: item.cost,
                units: item.units,
                weight: item.weight,
                hs_code: item.hs_code  // Convert to snake_case
            }));*/
    
            //console.log("Final payload:", payload); // Verify before sending
    
            const response = await fetch('/process-pdf', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(items)
            });
            
            if (!response.ok) {
                const errorData = await response.json().catch(() => ({}));
                throw new Error('PDF processing failed');
            }

            const result = await response.json();
            if (!result.success) {
                throw new Error(result.message);
            }
                
            
            // Show the PDF preview
            const timestamp = new Date().getTime();
            document.getElementById('pdf-preview').src = `/download?${timestamp}`;
            document.getElementById('pdf-result').classList.remove('d-none');
            document.getElementById('download-btn').href = '/download';
            
            // Scroll to the result
            document.getElementById('pdf-result').scrollIntoView();
        } catch (error) {
            console.error("PDF generation error:", error);
            alert(`Error: ${error.message}`);
        } finally {
            document.getElementById('loading').style.display = 'none';
        }
    });

});
    